# Material sampling on Metal — 2026-09-10

Returning early for absent material texture pages reduces measured offscreen
elapsed time by 0.7–1.1% on the tested M3 Pro workloads at 720p and 1080p.
Shared goldens remain unchanged. This is a small measured improvement, not a
claim about every scene, GPU or backend.

## Change

The forward shader previously sampled base-color, metallic/roughness/occlusion
and emissive textures even when a material named `NO_PAGE`, then discarded the
sample. That kept implicit-LOD texture operations in uniform control flow for
WGSL validation.

Each helper now computes UV derivatives before its material-dependent return and
uses `SampleGrad` only when a page exists. The missing-page value remains exact
white RGBA. This follows the existing normal-map path and preserves resource
bindings, physical/authored UV selection and material factors. Forward shading,
masked depth and reflective shadow maps share the corrected helpers.

## Method and results

- Apple M3 Pro, 18 GPU cores, macOS 26.5.2; Rust 1.97.0 release builds.
- Offscreen Metal, renderer geometry `IndirectPerBatch`, validation disabled,
  CPU tracing enabled. No concurrent local GPU tests or compilation.
- Sandbox draws its deterministic rotating cube. Sundial adds a plaza and
  scripted moving sun with PCSS shadows. Lantern includes textured surfaces, a
  live second-view monitor and volumetric fog.
- One complete warmup round was excluded. Six measured paired rounds followed,
  with three before-first and three after-first rounds. The sixth measured round
  balanced the initially planned five rounds after order effects were observed.
- The table reports median whole-process wall time from a monotonic host clock.
  The headless simulation's fixed 60 Hz clock is not a performance measurement.

| Workload | Viewport  | Frames/run | Before median | After median | Less elapsed time |
| -------- | --------- | ---------- | ------------- | ------------ | ----------------- |
| sandbox  | 1280x720  | 2400       | 4.8798 s      | 4.8448 s     | 0.72%             |
| sandbox  | 1920x1080 | 2400       | 9.5549 s      | 9.4854 s     | 0.73%             |
| sundial  | 1280x720  | 2400       | 6.5355 s      | 6.4796 s     | 0.85%             |
| sundial  | 1920x1080 | 2400       | 13.6454 s     | 13.4988 s    | 1.07%             |
| lantern  | 1280x720  | 1200       | 4.8154 s      | 4.7807 s     | 0.72%             |
| lantern  | 1920x1080 | 1200       | 9.1414 s      | 9.0440 s     | 1.07%             |

All 36 measured pairs completed successfully and were non-regressing; one
Lantern 720p pair was effectively flat. Median paired reductions are 0.67–1.08%.
An auxiliary interval from the first display-timing report to the GPU-pass
summary gives similar median reductions (0.68–1.09%); the JSON preserves both
timestamps. This excludes initialization and teardown, supporting a benefit
beyond startup. These are end-to-end timings, not isolated fragment-kernel
measurements.

[Raw paired samples and binary/source hashes](metal-material-sampling-results.json)
record every warmup and measured sample. The before and after code commits are
`c02540aa` and `3ecad8c4`; these measurements precede native mesh-path
enablement.

```sh
cargo build --locked --release -p sandbox -p sundial -p lantern
CRCBL_TRACE=1 MTL_DEBUG_LAYER=0 MTL_SHADER_VALIDATION=0 \
  <preserved-binary> --headless --backend mtl --frames 2400 \
  --size 1280x720 --fps 0 --no-debug-overlay
```

Use 1200 frames for Lantern and repeat at 1920x1080. Preserve separate before
and after binaries and alternate their order as described above.

## Correctness and portability

- Pinned Slang 2026.14, DXC 1.9 and SPIRV-Tools 2026.1 regenerated all required
  targets; the official byte-for-byte check passed
  ([run 34468251567](https://github.com/digitaloten/crcbl/actions/runs/34468251567)).
  Only the expected seven mesh artifacts/manifest entries changed.
- Naga's three WGSL checks passed. Headless Chrome/Dawn accepted both baseline
  and candidate modules and rejected a deliberately non-uniform derivative
  control. This checks the uniformity rule Naga does not enforce here.
- Apple's Metal compiler accepted the changed MSL artifact.
- Strict Metal validation passed 59 renderer goldens, all 93 existing mesh
  checks, and 37 forward checks. API errors/warnings asserted; shader reporting,
  stderr reporting and abort-on-fault were enabled.
- A new minification regression passed on both shaders. Base-color, MRO and
  emissive checker pages exactly matched their averaged HDR references; their
  magnified controls retained strong contrast. It detects lost mip selection,
  while existing material/cutout tests cover the broader shading behavior.
- Lantern's seven image/lighting checks passed under strict validation,
  including its live monitor and presentation-size image. Its separate
  lower-path test fails identically on the baseline: it treats native capability
  flags as selected renderer paths. That path-forcing defect is tracked for the
  native mesh/control qualification; no test or CI exclusion was added for it.
- Rust 1.97 Clippy with warnings denied and formatting checks passed.
