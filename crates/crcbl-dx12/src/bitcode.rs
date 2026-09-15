//! The structured buffers a DXIL container declares, read out of its LLVM
//! bitcode — each one's register, space and **element stride**.
//!
//! Like `crate::dxil`, this holds no `windows` type, so everything it answers is
//! proven against the committed containers on any host.
//!
//! # Why the stride has to be read at all
//!
//! Slang emits a storage buffer as `StructuredBuffer<T>`, and a D3D12 driver
//! addresses a structured buffer's elements by the **view's**
//! `StructureByteStride`. A raw view — `R32_TYPELESS` with
//! `D3D12_BUFFER_SRV_FLAG_RAW`, stride zero — is what `ByteBuffer` takes, and
//! bound under a `StructuredBuffer<T>` every element aliases element zero on
//! hardware. WARP tolerates it, which is how every storage buffer on this
//! backend was bound that way while CI stayed green; an RX 9060 XT drew nothing
//! at all. The seam therefore carries each storage buffer's element stride, and
//! this module is how the backend checks that number against the shader, the
//! way `crate::dxil` checks `ComputePipelineDesc::workgroup_size` against
//! `[numthreads]`.
//!
//! # What is read, and nothing more
//!
//! DXIL is LLVM 3.7 bitcode, and the one place a structured buffer's stride is
//! recorded is the module-level named metadata `!dx.resources`:
//!
//! ```text
//! !dx.resources = !{!{srvs, uavs, cbuffers, samplers}}
//! srv = !{id, gv, name, space, lower bound, range size, kind, sample count, tags}
//! uav = !{id, gv, name, space, lower bound, range size, kind, …, tags}
//! tags = !{tag, value, tag, value, …}   ; tag 1 is the structured-buffer stride
//! ```
//!
//! Reaching it takes four things and this module implements exactly those: the
//! bitstream itself (abbreviations, `BLOCKINFO`, blobs), module-level value
//! numbering (globals, functions and aliases before the module's constants), the
//! integer constants, and module-level metadata records. Function bodies, types
//! and debug information are skipped by their block length, never decoded.
//!
//! The layouts are the formats' own — LLVM's bitstream and 3.7 record codes, and
//! the DXIL metadata schema in `DxilMetadataHelper` — fixed outside this
//! repository. Every committed container is parsed by this module's tests.

use std::collections::HashMap;

/// `DxilProgramHeader` is eight bytes, then `DxilBitcodeHeader`'s magic,
/// version, offset and size.
const BITCODE_HEADER: usize = 8;

/// The four-byte bitcode magic, `BC` `0xC0DE`.
const BITCODE_MAGIC: [u8; 4] = [b'B', b'C', 0xC0, 0xDE];

/// Abbreviation ids every block reserves.
const END_BLOCK: u64 = 0;
const ENTER_SUBBLOCK: u64 = 1;
const DEFINE_ABBREV: u64 = 2;
const UNABBREV_RECORD: u64 = 3;

/// Block ids.
const BLOCKINFO_BLOCK: u64 = 0;
const MODULE_BLOCK: u64 = 8;
const CONSTANTS_BLOCK: u64 = 11;
const METADATA_BLOCK: u64 = 15;

/// `BLOCKINFO`'s one record this module needs.
const BLOCKINFO_SETBID: u64 = 1;

/// Module records that each define one module-level value.
const MODULE_GLOBALVAR: u64 = 7;
const MODULE_FUNCTION: u64 = 8;
const MODULE_ALIAS_OLD: u64 = 9;
const MODULE_ALIAS: u64 = 14;

/// Constants records: every one but `SETTYPE` defines a value.
const CONSTANT_SETTYPE: u64 = 1;
/// The zero of whatever type is current. LLVM writes `i32 0` this way rather
/// than as an `INTEGER`, so every register, space and resource id of zero is
/// one of these.
const CONSTANT_NULL: u64 = 2;
const CONSTANT_INTEGER: u64 = 4;

/// Metadata records, as LLVM 3.7 numbers them.
const METADATA_STRING: u64 = 1;
const METADATA_VALUE: u64 = 2;
const METADATA_NODE: u64 = 3;
const METADATA_NAME: u64 = 4;
const METADATA_DISTINCT_NODE: u64 = 5;
const METADATA_KIND: u64 = 6;
const METADATA_NAMED_NODE: u64 = 10;
const METADATA_ATTACHMENT: u64 = 11;

/// `DXIL::ResourceKind::StructuredBuffer`.
const KIND_STRUCTURED_BUFFER: i64 = 12;

/// `kDxilStructuredBufferElementStrideTag`.
const TAG_STRUCTURED_STRIDE: i64 = 1;

/// Which register class a structured buffer occupies.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum BufferClass {
    /// `StructuredBuffer<T>`, a `t` register.
    ShaderResource,
    /// `RWStructuredBuffer<T>`, a `u` register.
    UnorderedAccess,
}

/// One structured buffer a container declares.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct StructuredBuffer {
    /// The HLSL name, for messages.
    pub(crate) name: String,
    pub(crate) class: BufferClass,
    /// `registerN(…, spaceM)`'s `M`.
    pub(crate) space: u32,
    /// The first register the declaration occupies.
    pub(crate) register: u32,
    /// Bytes per element — `sizeof(T)` as DXIL lays it out.
    pub(crate) stride: u32,
}

/// Every structured buffer the `DXIL` part starting at `data` declares.
///
/// # Errors
///
/// A sentence naming what was malformed or missing: a truncated bitstream, an
/// abbreviation this format does not define, a `!dx.resources` whose shape is
/// not the schema's, or a structured buffer with no stride tag.
pub(crate) fn structured_buffers(part: &[u8]) -> Result<Vec<StructuredBuffer>, String> {
    let offset = word(part, BITCODE_HEADER + 8)
        .ok_or("the DXIL part is too short to hold its bitcode header")? as usize;
    let size = word(part, BITCODE_HEADER + 12)
        .ok_or("the DXIL part is too short to hold its bitcode header")? as usize;
    let start = BITCODE_HEADER
        .checked_add(offset)
        .ok_or("the DXIL part's bitcode offset overflows")?;
    let bitcode = start
        .checked_add(size)
        .and_then(|end| part.get(start..end))
        .ok_or("the DXIL part's bitcode runs past the end of the part")?;
    if bitcode.get(..4) != Some(&BITCODE_MAGIC) {
        return Err("the DXIL part's bitcode does not open with the BC 0xC0DE magic".into());
    }

    let mut module = Module::default();
    let mut reader = Reader {
        bits: Bits {
            bytes: bitcode,
            position: 32,
        },
        blockinfo: HashMap::new(),
    };
    reader.top_level(&mut module)?;
    module.resources()
}

/// One little-endian word at `at`, or `None` past the end.
fn word(bytes: &[u8], at: usize) -> Option<u32> {
    bytes
        .get(at..at.checked_add(4)?)
        .map(|four| u32::from_le_bytes([four[0], four[1], four[2], four[3]]))
}

/// Bits read least-significant first, which is the bitstream's order.
struct Bits<'a> {
    bytes: &'a [u8],
    /// The next bit to read, counted from the start of `bytes`.
    position: usize,
}

impl Bits<'_> {
    /// `width` bits as an unsigned value. Zero bits is a legal width and reads
    /// zero.
    fn fixed(&mut self, width: u32) -> Result<u64, String> {
        if width > 64 {
            return Err(format!("a {width}-bit field is wider than 64 bits"));
        }
        let mut value = 0_u64;
        for bit in 0..width {
            let byte = self
                .bytes
                .get(self.position / 8)
                .ok_or("the bitstream ends inside a field")?;
            value |= u64::from((byte >> (self.position % 8)) & 1) << bit;
            self.position += 1;
        }
        Ok(value)
    }

    /// A variable-width value in `width`-bit chunks, whose top bit says another
    /// chunk follows.
    fn vbr(&mut self, width: u32) -> Result<u64, String> {
        if width < 2 {
            return Err(format!(
                "a VBR{width} field has no room for a continuation bit"
            ));
        }
        let continuation = 1_u64 << (width - 1);
        let mut value = 0_u64;
        let mut shift = 0_u32;
        loop {
            let chunk = self.fixed(width)?;
            let bits = chunk & (continuation - 1);
            if shift >= 64 || (shift > 0 && bits >> (64 - shift) != 0) {
                return Err("a VBR value overflows 64 bits".into());
            }
            value |= bits << shift;
            if chunk & continuation == 0 {
                return Ok(value);
            }
            shift += width - 1;
        }
    }

    /// Skips to the next 32-bit boundary.
    fn align(&mut self) {
        self.position = self.position.next_multiple_of(32);
    }

    fn at_end(&self) -> bool {
        self.position >= self.bytes.len() * 8
    }
}

/// One operand of an abbreviation.
#[derive(Clone, Debug)]
enum Operand {
    Literal(u64),
    Fixed(u32),
    Vbr(u32),
    /// A length, then that many of the element operand.
    Array(Box<Operand>),
    Char6,
    /// A length, alignment, that many bytes, alignment.
    Blob,
}

type Abbreviation = Vec<Operand>;

/// The bitstream walker: the bits, and the abbreviations `BLOCKINFO` gave each
/// block id.
struct Reader<'a> {
    bits: Bits<'a>,
    blockinfo: HashMap<u64, Vec<Abbreviation>>,
}

/// One record: its code and its operands.
struct Record {
    code: u64,
    operands: Vec<u64>,
}

/// What a block's body produced at one step.
enum Entry {
    End,
    Block {
        id: u64,
        abbrev_width: u32,
        words: usize,
    },
    Record(Record),
}

impl Reader<'_> {
    /// The stream's top level, where the module block lives.
    fn top_level(&mut self, module: &mut Module) -> Result<(), String> {
        let mut abbreviations = Vec::new();
        while !self.bits.at_end() {
            // Trailing padding after the last block is zero bits, which reads as
            // an END_BLOCK at top level; there is nothing after it.
            if self.bits.bytes.len() * 8 - self.bits.position < 32 {
                break;
            }
            match self.entry(2, &mut abbreviations)? {
                Entry::End => break,
                Entry::Block {
                    id, abbrev_width, ..
                } if id == MODULE_BLOCK => self.module(abbrev_width, module)?,
                Entry::Block {
                    id, abbrev_width, ..
                } if id == BLOCKINFO_BLOCK => self.blockinfo(abbrev_width)?,
                Entry::Block { words, .. } => self.skip(words)?,
                Entry::Record(_) => {}
            }
        }
        Ok(())
    }

    /// Reads the next abbreviation id and whatever it introduces. A definition
    /// is added to `abbreviations` and the step repeats.
    fn entry(
        &mut self,
        width: u32,
        abbreviations: &mut Vec<Abbreviation>,
    ) -> Result<Entry, String> {
        loop {
            let id = self.bits.fixed(width)?;
            match id {
                END_BLOCK => {
                    self.bits.align();
                    return Ok(Entry::End);
                }
                ENTER_SUBBLOCK => {
                    let id = self.bits.vbr(8)?;
                    let abbrev_width = u32::try_from(self.bits.vbr(4)?)
                        .map_err(|_| "an abbreviation width does not fit 32 bits")?;
                    self.bits.align();
                    let words = self.bits.fixed(32)? as usize;
                    return Ok(Entry::Block {
                        id,
                        abbrev_width,
                        words,
                    });
                }
                DEFINE_ABBREV => abbreviations.push(self.define_abbreviation()?),
                UNABBREV_RECORD => {
                    let code = self.bits.vbr(6)?;
                    let count = self.bits.vbr(6)?;
                    let mut operands = Vec::new();
                    for _ in 0..count {
                        operands.push(self.bits.vbr(6)?);
                    }
                    return Ok(Entry::Record(Record { code, operands }));
                }
                defined => {
                    let index = usize::try_from(defined - 4)
                        .map_err(|_| "an abbreviation id does not fit an index")?;
                    let abbreviation = abbreviations
                        .get(index)
                        .cloned()
                        .ok_or_else(|| format!("abbreviation {defined} was never defined"))?;
                    return self.abbreviated(&abbreviation).map(Entry::Record);
                }
            }
        }
    }

    fn define_abbreviation(&mut self) -> Result<Abbreviation, String> {
        let count = self.bits.vbr(5)?;
        let mut operands = Vec::new();
        let mut remaining = count;
        while remaining > 0 {
            remaining -= 1;
            let operand = self.operand_definition()?;
            if matches!(operand, Operand::Array(_)) {
                // The array's element is the next definition and counts against
                // the same total.
                if remaining == 0 {
                    return Err("an array abbreviation has no element operand".into());
                }
                remaining -= 1;
                let element = self.operand_definition()?;
                operands.push(Operand::Array(Box::new(element)));
            } else {
                operands.push(operand);
            }
        }
        Ok(operands)
    }

    fn operand_definition(&mut self) -> Result<Operand, String> {
        if self.bits.fixed(1)? == 1 {
            return Ok(Operand::Literal(self.bits.vbr(8)?));
        }
        match self.bits.fixed(3)? {
            1 => Ok(Operand::Fixed(
                u32::try_from(self.bits.vbr(5)?).map_err(|_| "a fixed width overflows")?,
            )),
            2 => Ok(Operand::Vbr(
                u32::try_from(self.bits.vbr(5)?).map_err(|_| "a VBR width overflows")?,
            )),
            // The element is filled in by `define_abbreviation`.
            3 => Ok(Operand::Array(Box::new(Operand::Literal(0)))),
            4 => Ok(Operand::Char6),
            5 => Ok(Operand::Blob),
            other => Err(format!(
                "abbreviation operand encoding {other} is not one the format defines"
            )),
        }
    }

    fn scalar(&mut self, operand: &Operand) -> Result<u64, String> {
        match operand {
            Operand::Literal(value) => Ok(*value),
            Operand::Fixed(width) => self.bits.fixed(*width),
            Operand::Vbr(width) => self.bits.vbr(*width),
            Operand::Char6 => self.bits.fixed(6),
            Operand::Array(_) | Operand::Blob => {
                Err("an array or blob cannot be an array's element".into())
            }
        }
    }

    fn abbreviated(&mut self, abbreviation: &[Operand]) -> Result<Record, String> {
        let mut values = Vec::new();
        for operand in abbreviation {
            match operand {
                Operand::Array(element) => {
                    let length = self.bits.vbr(6)?;
                    for _ in 0..length {
                        values.push(self.scalar(element)?);
                    }
                }
                Operand::Blob => {
                    let length = self.bits.vbr(6)?;
                    self.bits.align();
                    for _ in 0..length {
                        values.push(self.bits.fixed(8)?);
                    }
                    self.bits.align();
                }
                scalar => values.push(self.scalar(scalar)?),
            }
        }
        if values.is_empty() {
            return Err("an abbreviated record has no code".into());
        }
        let code = values.remove(0);
        Ok(Record {
            code,
            operands: values,
        })
    }

    /// Skips a block's body by its declared length.
    fn skip(&mut self, words: usize) -> Result<(), String> {
        let end = words
            .checked_mul(32)
            .and_then(|bits| self.bits.position.checked_add(bits))
            .ok_or("a block length overflows")?;
        if end > self.bits.bytes.len() * 8 {
            return Err("a block's declared length runs past the end of the bitcode".into());
        }
        self.bits.position = end;
        Ok(())
    }

    /// The abbreviations a new block of `id` starts with.
    fn inherited(&self, id: u64) -> Vec<Abbreviation> {
        self.blockinfo.get(&id).cloned().unwrap_or_default()
    }

    fn blockinfo(&mut self, width: u32) -> Result<(), String> {
        let mut current: Option<u64> = None;
        loop {
            let id = self.bits.fixed(width)?;
            match id {
                END_BLOCK => {
                    self.bits.align();
                    return Ok(());
                }
                ENTER_SUBBLOCK => {
                    let _ = self.bits.vbr(8)?;
                    let _ = self.bits.vbr(4)?;
                    self.bits.align();
                    let words = self.bits.fixed(32)? as usize;
                    self.skip(words)?;
                }
                DEFINE_ABBREV => {
                    let abbreviation = self.define_abbreviation()?;
                    let target =
                        current.ok_or("BLOCKINFO defines an abbreviation before SETBID")?;
                    self.blockinfo.entry(target).or_default().push(abbreviation);
                }
                UNABBREV_RECORD => {
                    let code = self.bits.vbr(6)?;
                    let count = self.bits.vbr(6)?;
                    let mut operands = Vec::new();
                    for _ in 0..count {
                        operands.push(self.bits.vbr(6)?);
                    }
                    if code == BLOCKINFO_SETBID {
                        current = Some(*operands.first().ok_or("SETBID names no block")?);
                    }
                }
                other => {
                    return Err(format!(
                        "BLOCKINFO uses abbreviation {other}, which it cannot define"
                    ));
                }
            }
        }
    }

    fn module(&mut self, width: u32, module: &mut Module) -> Result<(), String> {
        let mut abbreviations = self.inherited(MODULE_BLOCK);
        loop {
            match self.entry(width, &mut abbreviations)? {
                Entry::End => return Ok(()),
                Entry::Block {
                    id, abbrev_width, ..
                } if id == BLOCKINFO_BLOCK => self.blockinfo(abbrev_width)?,
                Entry::Block {
                    id, abbrev_width, ..
                } if id == CONSTANTS_BLOCK => self.constants(abbrev_width, module)?,
                Entry::Block {
                    id, abbrev_width, ..
                } if id == METADATA_BLOCK => self.metadata(abbrev_width, module)?,
                Entry::Block { words, .. } => self.skip(words)?,
                Entry::Record(record) => {
                    if matches!(
                        record.code,
                        MODULE_GLOBALVAR | MODULE_FUNCTION | MODULE_ALIAS_OLD | MODULE_ALIAS
                    ) {
                        module.values.push(None);
                    }
                }
            }
        }
    }

    fn constants(&mut self, width: u32, module: &mut Module) -> Result<(), String> {
        let mut abbreviations = self.inherited(CONSTANTS_BLOCK);
        loop {
            match self.entry(width, &mut abbreviations)? {
                Entry::End => return Ok(()),
                Entry::Block { words, .. } => self.skip(words)?,
                Entry::Record(record) => match record.code {
                    CONSTANT_SETTYPE => {}
                    // Zero for an integer; for a pointer or aggregate the value
                    // is never read as a number, so zero is harmless there.
                    CONSTANT_NULL => module.values.push(Some(0)),
                    CONSTANT_INTEGER => {
                        let encoded = *record
                            .operands
                            .first()
                            .ok_or("an integer constant has no value")?;
                        module.values.push(Some(signed(encoded)));
                    }
                    _ => module.values.push(None),
                },
            }
        }
    }

    fn metadata(&mut self, width: u32, module: &mut Module) -> Result<(), String> {
        let mut abbreviations = self.inherited(METADATA_BLOCK);
        let mut name: Option<String> = None;
        loop {
            match self.entry(width, &mut abbreviations)? {
                Entry::End => return Ok(()),
                Entry::Block { words, .. } => self.skip(words)?,
                Entry::Record(record) => match record.code {
                    METADATA_STRING => module
                        .metadata
                        .push(Metadata::String(text(&record.operands))),
                    METADATA_VALUE => {
                        let value = *record
                            .operands
                            .get(1)
                            .ok_or("a metadata value names no value")?;
                        module.metadata.push(Metadata::Value(value));
                    }
                    METADATA_NODE | METADATA_DISTINCT_NODE => module.metadata.push(Metadata::Node(
                        record
                            .operands
                            .iter()
                            .map(|operand| operand.checked_sub(1))
                            .collect(),
                    )),
                    METADATA_NAME => name = Some(text(&record.operands)),
                    METADATA_NAMED_NODE => {
                        if name.take().as_deref() == Some("dx.resources") {
                            module.resources = Some(record.operands);
                        }
                    }
                    METADATA_KIND | METADATA_ATTACHMENT => {}
                    // Everything else — a location, a debug-info node — is one
                    // metadata id this module never reads.
                    _ => module.metadata.push(Metadata::Other),
                },
            }
        }
    }
}

/// A signed VBR constant: the sign in the low bit, the magnitude above it.
fn signed(encoded: u64) -> i64 {
    let magnitude = (encoded >> 1).cast_signed();
    if encoded & 1 == 0 {
        magnitude
    } else if magnitude == 0 {
        i64::MIN
    } else {
        -magnitude
    }
}

/// A record's operands as text, one character each.
fn text(operands: &[u64]) -> String {
    operands
        .iter()
        .map(|&unit| char::from(u8::try_from(unit).unwrap_or(b'?')))
        .collect()
}

/// One metadata id's contents.
enum Metadata {
    String(String),
    /// A module-level value id.
    Value(u64),
    /// Operands as metadata ids; `None` is a null operand.
    Node(Vec<Option<u64>>),
    Other,
}

/// What the module walk collected.
#[derive(Default)]
struct Module {
    /// Every module-level value in numbering order: `Some` for an integer
    /// constant, `None` for anything else.
    values: Vec<Option<i64>>,
    metadata: Vec<Metadata>,
    /// `!dx.resources`' operands, as metadata ids.
    resources: Option<Vec<u64>>,
}

impl Module {
    fn node(&self, id: Option<u64>) -> Result<Option<&[Option<u64>]>, String> {
        let Some(id) = id else { return Ok(None) };
        match self.entry(id)? {
            Metadata::Node(operands) => Ok(Some(operands)),
            _ => Err(format!("metadata {id} is not a node")),
        }
    }

    fn entry(&self, id: u64) -> Result<&Metadata, String> {
        usize::try_from(id)
            .ok()
            .and_then(|index| self.metadata.get(index))
            .ok_or_else(|| format!("metadata {id} is referenced and never defined"))
    }

    fn integer(&self, operand: Option<&Option<u64>>) -> Result<i64, String> {
        let id = operand
            .copied()
            .flatten()
            .ok_or("a resource field that must be an integer is missing")?;
        let Metadata::Value(value) = self.entry(id)? else {
            return Err(format!("metadata {id} is not a value"));
        };
        usize::try_from(*value)
            .ok()
            .and_then(|index| self.values.get(index))
            .copied()
            .flatten()
            .ok_or_else(|| format!("value {value} is not an integer constant"))
    }

    fn string(&self, operand: Option<&Option<u64>>) -> Result<String, String> {
        match operand
            .copied()
            .flatten()
            .map(|id| self.entry(id))
            .transpose()?
        {
            Some(Metadata::String(text)) => Ok(text.clone()),
            _ => Ok(String::from("<unnamed>")),
        }
    }

    fn resources(&self) -> Result<Vec<StructuredBuffer>, String> {
        let Some(named) = &self.resources else {
            // A container that binds nothing carries no `!dx.resources` at all.
            return Ok(Vec::new());
        };
        let root = self
            .node(named.first().copied())?
            .ok_or("!dx.resources names no node")?;
        let mut found = Vec::new();
        for (list, class) in [
            (root.first(), BufferClass::ShaderResource),
            (root.get(1), BufferClass::UnorderedAccess),
        ] {
            let Some(list) = self.node(list.copied().flatten())? else {
                continue;
            };
            for resource in list {
                let fields = self
                    .node(*resource)?
                    .ok_or("a resource list holds a null resource")?;
                if self.integer(fields.get(6))? != KIND_STRUCTURED_BUFFER {
                    continue;
                }
                let name = self.string(fields.get(2))?;
                // **The last field, not a fixed index.** The tag list closes
                // every resource record, but how many fields precede it has
                // grown across DXIL versions — a UAV's sample count arrived
                // after its counter and ROV flags — so a hard-coded position is
                // right for one validator and wrong for the next.
                let tags = self
                    .node(fields.last().copied().flatten())?
                    .ok_or_else(|| {
                        format!("structured buffer `{name}` carries no tags, so no stride")
                    })?;
                let mut stride = None;
                for pair in tags.chunks(2) {
                    if self.integer(pair.first())? == TAG_STRUCTURED_STRIDE {
                        stride = Some(self.integer(pair.get(1))?);
                    }
                }
                let stride = stride
                    .ok_or_else(|| format!("structured buffer `{name}` carries no stride tag"))?;
                let field = |value: i64, what: &str| {
                    u32::try_from(value)
                        .map_err(|_| format!("structured buffer `{name}`'s {what} is {value}"))
                };
                found.push(StructuredBuffer {
                    class,
                    space: field(self.integer(fields.get(3))?, "space")?,
                    register: field(self.integer(fields.get(4))?, "register")?,
                    stride: field(stride, "stride")?,
                    name,
                });
            }
        }
        Ok(found)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The `DXIL` part of a whole container.
    fn dxil_part(container: &[u8]) -> &[u8] {
        let count = word(container, 28).expect("a part count") as usize;
        for part in 0..count {
            let offset = word(container, 32 + part * 4).expect("a part offset") as usize;
            if container.get(offset..offset + 4) == Some(b"DXIL") {
                let size = word(container, offset + 4).expect("a part size") as usize;
                return &container[offset + 8..offset + 8 + size];
            }
        }
        panic!("the container has no DXIL part");
    }

    /// **Every committed container parses**, and every structured buffer in it
    /// has a stride that is a whole, non-zero number of four-byte words — HLSL
    /// packs a structured buffer's element to a four-byte boundary, so any other
    /// figure is the reader misplacing a field.
    #[test]
    fn every_committed_container_yields_its_structured_buffers() {
        let mut containers = 0;
        let mut buffers = 0;
        for shader in crcbl_shaders::ALL {
            for (entry, bytes) in shader.dxil_containers() {
                containers += 1;
                let found = structured_buffers(dxil_part(bytes))
                    .unwrap_or_else(|error| panic!("{}:{entry}: {error}", shader.name()));
                for buffer in &found {
                    assert!(
                        buffer.stride > 0 && buffer.stride.is_multiple_of(4),
                        "{}:{entry}: {buffer:?}",
                        shader.name()
                    );
                }
                buffers += found.len();
            }
        }
        assert!(containers > 0, "no container was read");
        assert!(
            buffers > 0,
            "no structured buffer was found in any container"
        );
    }

    /// The figures, on artifacts whose element types are known from their
    /// sources, so a reader that returned plausible multiples of four from the
    /// wrong field fails here.
    ///
    /// `triangle.slang` pulls `StructuredBuffer<Vertex>`, two `float4`s — 32
    /// bytes, which `crcbl_shaders::triangle::VERTEX_STRIDE` publishes — at `t0`
    /// in the vertex container and nothing in the fragment one. The compute
    /// probe writes an `RWStructuredBuffer<uint>`, four bytes, at `u0`.
    #[test]
    fn the_known_artifacts_report_the_strides_their_sources_declare() {
        let vertex = crcbl_shaders::TRIANGLE
            .dxil_containers()
            .into_iter()
            .find(|(entry, _)| *entry == "vertexMain")
            .map(|(_, bytes)| bytes)
            .expect("triangle commits a vertex container");
        let found = structured_buffers(dxil_part(vertex)).expect("the triangle vertex container");
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].class, BufferClass::ShaderResource);
        assert_eq!((found[0].space, found[0].register), (0, 0));
        assert_eq!(
            found[0].stride as usize,
            crcbl_shaders::triangle::VERTEX_STRIDE,
            "{found:?}"
        );

        let probe = crcbl_shaders::COMPUTE_PROBE
            .dxil_containers()
            .into_iter()
            .next()
            .map(|(_, bytes)| bytes)
            .expect("the compute probe commits a container");
        let found = structured_buffers(dxil_part(probe)).expect("the compute probe container");
        let written: Vec<_> = found
            .iter()
            .filter(|buffer| buffer.class == BufferClass::UnorderedAccess)
            .collect();
        assert!(!written.is_empty(), "{found:?}");
        assert!(written.iter().all(|buffer| buffer.stride == 4), "{found:?}");
    }

    #[test]
    fn a_signed_vbr_constant_decodes_both_signs_and_the_minimum() {
        assert_eq!(signed(0), 0);
        assert_eq!(signed(2 << 1), 2);
        assert_eq!(signed((2 << 1) | 1), -2);
        assert_eq!(signed(1), i64::MIN);
    }

    #[test]
    fn a_part_that_is_not_bitcode_is_refused_by_name() {
        let mut part = vec![0_u8; 32];
        part[8..12].copy_from_slice(b"DXIL");
        part[16..20].copy_from_slice(&16_u32.to_le_bytes());
        part[20..24].copy_from_slice(&8_u32.to_le_bytes());
        let error = structured_buffers(&part).expect_err("zeros are not bitcode");
        assert!(error.contains("magic"), "{error}");
    }
}
