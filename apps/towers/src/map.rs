//! The one map: the field the creeps cross, the lane they walk, the pads a
//! tower can be built on, and the volume that takes a life off the team.
//!
//! ```text
//!            +X
//!             │   ┌─────────────────────────────────────────────┐  z = -12
//!             │   │                                             │
//!             │   │   ▣ exit  ◀───────────────────── leg 2      │  z =  -6
//!             │   │       ▫ gate      ▫ middle          ▲       │
//!             │   │                                     │ leg 1 │
//!             │   │       ▫ entry     ▫ bend      ▫ east│       │  z =   1..3
//!             │   │   ▲ spawn ──────────────────────────▶       │  z =   8
//!             │   └─────────────────────────────────────────────┘  z = +12
//!            −X       x = -14                            x = +13
//! ```
//!
//! # One set of numbers, three consumers
//!
//! Every constant here is read by the meshes in [`scene`], by the instances in
//! [`place`], and by the colliders in [`world`]. There is no second set of
//! numbers for the physics, which is what makes a lane that looks walkable
//! walkable and an exit volume that looks like a gate the gate. `apps/breach`
//! and `apps/shard` build their rooms the same way, and for the same reason.
//!
//! # The map is a table in this file, and milestone 2 is where it stops being
//!
//! `docs/plan/sample/07-towers.md` asks for a map authored in the stage 8
//! editor and shipped as a `.scn/` directory. There is no `apps/editor`, so
//! this slice hardcodes the map the way that document's milestone 1 says to.
//! Nothing here reads a file.
//!
//! # The creeps are not in [`world`]
//!
//! Each adds its own sphere when it spawns and writes it back every tick — see
//! [`crate::creep`]. What this module puts in the world is what does not move:
//! the ground and the exit trigger.
//!
//! # The exit is a trigger, and that is the whole of how a life is lost
//!
//! [`EXIT_HALF`] is registered with
//! [`PhysicsWorld::set_trigger`](crcbl::phys::PhysicsWorld::set_trigger), which
//! makes it **non-solid**: every sweep and every ray passes straight through it
//! and only [`overlap_sphere`](crcbl::phys::PhysicsWorld::overlap_sphere)
//! reports it. So a creep standing in it is found by the overlap query
//! [`crate::creep::has_reached_the_exit`] runs, and a bolt fired down the last
//! leg flies through it rather than exploding on it. Both halves are asserted —
//! see this module's tests.
//!
//! # There is a sun in here, because this map has no roof
//!
//! [`sun`] is a real [`DirectionalLight`] rather than the token
//! `apps/breach::map::house_light` hands its ceilinged room, and [`place`] sets
//! no point lights at all. A field under the sky is the one thing on the ladder
//! that wants exactly what `begin_frame` already takes.

use std::borrow::Cow;

use crcbl::greybox::{GREYBOX_TILE_M, cylinder, grid_material, grid_page, platform, sphere};
use crcbl::math::{DVec3, Mat4, Vec3};
use crcbl::phys::{BoxCollider, ColliderId, PhysicsWorld};
use crcbl::render::scene::{Capacities, Geometry, InstanceDesc, MeshDesc, ProbeGrid, SceneDesc};
use crcbl::render::{DirectionalLight, ForwardRenderer, InstanceHandle, InstancePoolError};
use crcbl::shaders::mesh::GpuMaterial;

use crate::wave::MAX_CREEPS;

// ---------------------------------------------------------------------------
// The field
// ---------------------------------------------------------------------------

/// How far the field reaches either side of its centre line, in metres.
pub const HALF_WIDTH: f64 = 18.0;

/// …and how far up and down it, in metres along `Z`.
pub const HALF_DEPTH: f64 = 12.0;

/// How thick the ground slab is, in metres. Its **top** is `y = 0`, which is
/// what every other height here is measured from.
pub const SLAB_THICKNESS: f64 = 0.6;

// ---------------------------------------------------------------------------
// The path
// ---------------------------------------------------------------------------

/// The waypoints the creeps walk, spawn first and exit last.
///
/// **A polyline, and deliberately not a spline.**
/// `docs/plan/sample/07-towers.md` asks for a spline follower and nothing in
/// `crcbl-phys` or `crcbl-scene` offers a spline type — the only splines in the
/// workspace are `crcbl-anim`'s clip interpolation and the glTF importer's — so
/// what [`crate::path`] walks is straight legs between these points. The
/// difference a spline would make is the corners, and the engine gap is
/// recorded in `docs/backlog.md` rather than worked around here.
///
/// Every leg is axis-aligned, which is what lets one `platform` per leg be both
/// the lane a reader sees and the length [`crate::path`] measures.
pub const PATH: [DVec3; 4] = [
    DVec3::new(-14.0, 0.0, 8.0),
    DVec3::new(8.0, 0.0, 8.0),
    DVec3::new(8.0, 0.0, -6.0),
    DVec3::new(-10.0, 0.0, -6.0),
];

/// How many straight legs [`PATH`] has.
pub const LEGS: usize = PATH.len() - 1;

/// How wide the lane is drawn, in metres. Decoration: nothing collides with it.
pub const LANE_WIDTH: f64 = 1.8;

/// How proud of the ground the lane stands, in metres.
pub const LANE_HEIGHT: f64 = 0.06;

// ---------------------------------------------------------------------------
// The exit
// ---------------------------------------------------------------------------

/// Half the exit volume's extent, in metres — a two-metre cube standing on the
/// last waypoint.
///
/// A creep whose sphere touches it costs the team a life. See the module docs
/// for why it is a trigger and what that buys.
pub const EXIT_HALF: DVec3 = DVec3::new(1.0, 1.0, 1.0);

/// Where the exit volume's centre is, in metres.
#[must_use]
pub fn exit_centre() -> DVec3 {
    let at = PATH[PATH.len() - 1];
    DVec3::new(at.x, EXIT_HALF.y, at.z)
}

/// The exit volume, as the physics world holds it.
#[must_use]
pub fn exit_collider() -> BoxCollider {
    BoxCollider::new(exit_centre(), EXIT_HALF)
}

// ---------------------------------------------------------------------------
// The build plots
// ---------------------------------------------------------------------------

/// One place a tower can be built.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plot {
    /// What the overlay, the debug panel and a failing test call it.
    pub label: &'static str,
    /// Where it stands across the field, in metres.
    pub x: f64,
    /// …and down it, in metres along `Z`.
    pub z: f64,
}

impl Plot {
    /// Where a tower on this plot has its feet, in metres.
    #[must_use]
    pub const fn at(&self) -> DVec3 {
        DVec3::new(self.x, 0.0, self.z)
    }
}

/// Every plot, in the order the overlay lists them and `PlaceTower` numbers
/// them.
///
/// Five, each beside a different stretch of [`PATH`] and none of them **on**
/// it: `every_plot_stands_clear_of_the_lane_and_still_covers_it` asserts both
/// the clearance and that each is close enough to reach the path, which is the
/// pair that says a plot is a build site rather than a decoration.
pub const PLOTS: [Plot; 5] = [
    Plot {
        label: "entry",
        x: -6.0,
        z: 3.0,
    },
    Plot {
        label: "bend",
        x: 3.0,
        z: 3.0,
    },
    Plot {
        label: "east",
        x: 13.0,
        z: 1.0,
    },
    Plot {
        label: "middle",
        x: 2.0,
        z: -1.0,
    },
    Plot {
        label: "gate",
        x: -7.0,
        z: -1.0,
    },
];

/// How wide a build pad is drawn, in metres.
pub const PAD_EDGE: f64 = 2.0;

/// How proud of the ground a build pad stands, in metres.
pub const PAD_HEIGHT: f64 = 0.08;

// ---------------------------------------------------------------------------
// What the pieces are, in metres
// ---------------------------------------------------------------------------

/// A creep's radius, in metres. **The collider's and the mesh's**, so what
/// looks shootable is shootable.
pub const CREEP_RADIUS: f64 = 0.45;

/// How wide a tower is, in metres.
pub const TOWER_RADIUS: f64 = 0.55;

/// How tall a tower stands, in metres.
pub const TOWER_HEIGHT: f64 = 2.0;

/// How high a tower's muzzle is, in metres — where every bolt starts.
pub const MUZZLE_Y: f64 = 1.6;

/// A bolt's radius, in metres. The **sweep's** radius as well as the mesh's.
pub const BOLT_RADIUS: f64 = 0.16;

/// How many bolts a frame draws: one slot per plot.
///
/// A pool rather than a count of what is in flight: instances are added once at
/// start-up and the unused ones are parked at [`PARK`], because adding and
/// removing an instance every frame would churn the pool for a thing that lives
/// a fraction of a second.
///
/// **One slot per plot is the real bound and not a guess.** A bolt is in the
/// air for less time than a tower takes to reload, so a tower never has two of
/// them out at once and a full field has one each: `crate::tower`'s
/// `a_bolt_lands_long_before_its_tower_reloads` asserts that timing on the
/// longest flight there is, and `crate::game`'s
/// `towers_clear_the_table_and_win_the_run` measures the peak over a whole run
/// against this pool. Bolts past it would be simulated and not drawn, which
/// would be a presentation limit and never a simulation one — those two tests
/// are what say the case does not arise.
pub const MAX_BOLTS: usize = PLOTS.len();

/// Where an unused instance is parked: under the ground slab, inside its
/// footprint, so the opaque floor hides it.
///
/// `the_parking_spot_is_under_the_ground` asserts both halves.
pub const PARK: DVec3 = DVec3::new(0.0, -6.0, 0.0);

// ---------------------------------------------------------------------------
// The scene description
// ---------------------------------------------------------------------------

/// The ground slab — [`SceneDesc::meshes`] slot 0.
pub const GROUND_MESH: usize = 0;
/// The first leg of the lane; leg `i` is `LANE_MESH + i`. One mesh per leg
/// because each is its own length, which is what keeps the lane the same
/// numbers [`crate::path`] measures.
pub const LANE_MESH: usize = 1;
/// A build pad.
pub const PAD_MESH: usize = LANE_MESH + LEGS;
/// The exit volume, drawn as the cube it is.
pub const EXIT_MESH: usize = PAD_MESH + 1;
/// A creep.
pub const CREEP_MESH: usize = EXIT_MESH + 1;
/// A tower.
pub const TOWER_MESH: usize = CREEP_MESH + 1;
/// A bolt in flight.
pub const BOLT_MESH: usize = TOWER_MESH + 1;
/// How many meshes this map makes resident.
pub const MESHES: usize = BOLT_MESH + 1;

/// The ground. [`SceneDesc::materials`] slot 0, and therefore what an instance
/// placed without a named material would shade through.
pub const GROUND_MATERIAL: usize = 0;
/// The lane the creeps walk.
pub const LANE_MATERIAL: usize = 1;
/// A build pad.
pub const PAD_MATERIAL: usize = 2;
/// The exit volume.
pub const EXIT_MATERIAL: usize = 3;
/// A creep at full health.
pub const CREEP_MATERIAL: usize = 4;
/// …and one that has been shot down to a third of it. **The picture says which**,
/// for the reason a knocked-down plate is drawn orange on breach's range: a
/// state a reviewer cannot see is a state they cannot check the readout against.
pub const CREEP_HURT_MATERIAL: usize = 5;
/// A tower between shots.
pub const TOWER_MATERIAL: usize = 6;
/// …and one that fired this tick.
pub const TOWER_FIRING_MATERIAL: usize = 7;
/// A bolt.
pub const BOLT_MATERIAL: usize = 8;
/// How many material rows this map declares.
pub const MATERIALS: usize = BOLT_MATERIAL + 1;

/// How many latitude bands and longitude columns a creep is drawn with, and how
/// many facets a tower's cylinder has.
///
/// Enough to read as a ball and a post from the overhead camera and no more:
/// this is a greybox field, and the browser the next slice publishes to is the
/// target.
const CREEP_RINGS: u32 = 8;
const CREEP_SEGMENTS: u32 = 14;
const TOWER_SEGMENTS: u32 = 12;
const BOLT_RINGS: u32 = 4;
const BOLT_SEGMENTS: u32 = 8;

/// What this map reserves, which is a little over what it places.
///
/// Sized against the description rather than left at [`Capacities::default`],
/// for `apps/breach/src/map.rs`'s reason: that default reserves far more
/// instances than this field needs, and the level-of-detail state behind that
/// number is a word per instance per draw generator. Filling any of these is a
/// mistake in this file rather than a condition a run can be in, and
/// `the_map_fits_the_pools_it_reserves` asserts it.
const CAPACITIES: Capacities = Capacities {
    vertices: 8 * 1024,
    indices: 16 * 1024,
    meshes: 12,
    instances: 64,
    materials: 12,
    lights: 4,
    probes: 0,
};

/// A painted greybox material: the metric grid of [`grid_page`], tinted, and
/// tiled **physically** so one tile measures [`GREYBOX_TILE_M`] of surface
/// however large the face is.
///
/// The tint is this map's own and the grid is the engine's. It spends the 32²
/// grid page rather than `crcbl::greybox::greybox_material`'s 1024² one,
/// because a demo that runs in a browser should not upload eight megatexels to
/// show a ruler. `apps/breach/src/map.rs` has the same helper for the same
/// reason.
fn painted(tint: [f32; 3]) -> GpuMaterial {
    GpuMaterial {
        base_color: [tint[0], tint[1], tint[2], 1.0],
        tiling: GpuMaterial::TILING_PHYSICAL,
        tile_metres: GREYBOX_TILE_M,
        ..grid_material()
    }
}

/// How wide and deep leg `leg` of the lane is drawn, in metres.
///
/// The leg's own length across whichever axis it runs, widened by
/// [`LANE_WIDTH`] on both — so the square end of one leg fills the corner the
/// next one turns out of and the lane reads as continuous.
fn lane_extent(leg: usize) -> (f64, f64) {
    let step = PATH[leg + 1] - PATH[leg];
    (step.x.abs() + LANE_WIDTH, step.z.abs() + LANE_WIDTH)
}

/// Everything this map makes resident: [`MESHES`] meshes, [`MATERIALS`] painted
/// rows and the grid page they sample.
///
/// The mesh and material order is the constants above, in value order; keep
/// them and this assembly in step, which `the_constants_name_their_own_meshes`
/// asserts.
#[must_use]
pub fn scene() -> SceneDesc<'static> {
    let mesh = |label: &'static str, geometry: Geometry<'static>| MeshDesc {
        label: Cow::Borrowed(label),
        geometry,
    };
    let mut meshes = Vec::with_capacity(MESHES);
    meshes.push(mesh(
        "ground",
        platform(
            2.0 * HALF_WIDTH as f32,
            2.0 * HALF_DEPTH as f32,
            SLAB_THICKNESS as f32,
        ),
    ));
    for leg in 0..LEGS {
        let (width, depth) = lane_extent(leg);
        meshes.push(mesh(
            "lane",
            platform(width as f32, depth as f32, LANE_HEIGHT as f32),
        ));
    }
    meshes.push(mesh(
        "pad",
        platform(PAD_EDGE as f32, PAD_EDGE as f32, PAD_HEIGHT as f32),
    ));
    meshes.push(mesh(
        "exit",
        platform(
            2.0 * EXIT_HALF.x as f32,
            2.0 * EXIT_HALF.z as f32,
            2.0 * EXIT_HALF.y as f32,
        ),
    ));
    meshes.push(mesh(
        "creep",
        sphere(CREEP_RADIUS as f32, CREEP_RINGS, CREEP_SEGMENTS),
    ));
    meshes.push(mesh(
        "tower",
        cylinder(TOWER_RADIUS as f32, TOWER_HEIGHT as f32, TOWER_SEGMENTS),
    ));
    meshes.push(mesh(
        "bolt",
        sphere(BOLT_RADIUS as f32, BOLT_RINGS, BOLT_SEGMENTS),
    ));

    SceneDesc {
        meshes,
        materials: vec![
            painted([0.26, 0.31, 0.24]),
            painted([0.46, 0.42, 0.32]),
            painted([0.30, 0.38, 0.46]),
            painted([0.72, 0.30, 0.28]),
            painted([0.55, 0.72, 0.40]),
            painted([0.86, 0.52, 0.24]),
            painted([0.58, 0.62, 0.70]),
            painted([0.95, 0.88, 0.45]),
            painted([0.98, 0.94, 0.60]),
        ],
        page: grid_page(),
        probes: ProbeGrid::default(),
        capacities: CAPACITIES,
    }
}

/// The instances a frame rewrites: one pool per moving thing.
///
/// The field itself — the ground, the lane, the pads and the exit — is placed
/// once and drawn for the rest of the run, so it is not in here.
#[derive(Debug)]
pub struct Field {
    creeps: [InstanceHandle; MAX_CREEPS],
    towers: [InstanceHandle; PLOTS.len()],
    bolts: [InstanceHandle; MAX_BOLTS],
}

/// Where a creep's mesh sits, given where its centre is.
fn creep_transform(centre: DVec3) -> Mat4 {
    Mat4::from_translation(Vec3::new(centre.x as f32, centre.y as f32, centre.z as f32))
}

impl Field {
    /// Draws one creep, or parks it under the ground when the pool is longer
    /// than the field is populated.
    ///
    /// # Panics
    ///
    /// If `index` is not in the pool. Called only from `crate::gpu`'s own
    /// enumeration of it.
    pub fn set_creep(
        &self,
        renderer: &mut ForwardRenderer,
        index: usize,
        view: Option<crate::creep::CreepView>,
    ) {
        let (material, centre) = match view {
            Some(view) if view.hurt => (CREEP_HURT_MATERIAL, view.centre),
            Some(view) => (CREEP_MATERIAL, view.centre),
            None => (CREEP_MATERIAL, PARK),
        };
        renderer.set_instance(
            self.creeps[index],
            &InstanceDesc {
                mesh: CREEP_MESH,
                material,
                transform: creep_transform(centre),
            },
        );
    }

    /// Draws one tower, or parks it on an empty plot.
    ///
    /// # Panics
    ///
    /// If `plot` is not a plot. Called only from `crate::gpu`'s own
    /// enumeration of [`PLOTS`].
    pub fn set_tower(&self, renderer: &mut ForwardRenderer, plot: usize, firing: Option<bool>) {
        let (material, at) = match firing {
            Some(true) => (TOWER_FIRING_MATERIAL, PLOTS[plot].at()),
            Some(false) => (TOWER_MATERIAL, PLOTS[plot].at()),
            None => (TOWER_MATERIAL, PARK),
        };
        renderer.set_instance(
            self.towers[plot],
            &InstanceDesc {
                mesh: TOWER_MESH,
                material,
                transform: Mat4::from_translation(Vec3::new(at.x as f32, at.y as f32, at.z as f32)),
            },
        );
    }

    /// Draws one bolt, or parks it under the ground.
    ///
    /// # Panics
    ///
    /// If `index` is not in the pool. Called only from `crate::gpu`'s own
    /// enumeration of it.
    pub fn set_bolt(&self, renderer: &mut ForwardRenderer, index: usize, at: Option<DVec3>) {
        renderer.set_instance(
            self.bolts[index],
            &InstanceDesc {
                mesh: BOLT_MESH,
                material: BOLT_MATERIAL,
                transform: creep_transform(at.unwrap_or(PARK)),
            },
        );
    }
}

/// Places the field and hands back the pools that move.
///
/// The lights are set here too, and they are sticky: nothing in this sample
/// moves the sun.
///
/// # Errors
///
/// [`InstancePoolError`] if `CAPACITIES`'s instance count does not cover the
/// map, which is this file's numbers being wrong rather than a condition a run
/// can be in.
pub fn place(renderer: &mut ForwardRenderer) -> Result<Field, InstancePoolError> {
    let at =
        |x: f64, y: f64, z: f64| Mat4::from_translation(Vec3::new(x as f32, y as f32, z as f32));

    // A `platform` rises from `y = 0`, so the ground is dropped by its own
    // thickness to put its top there.
    renderer.add_instance(&InstanceDesc {
        mesh: GROUND_MESH,
        material: GROUND_MATERIAL,
        transform: at(0.0, -SLAB_THICKNESS, 0.0),
    })?;

    for leg in 0..LEGS {
        let middle = 0.5 * (PATH[leg] + PATH[leg + 1]);
        renderer.add_instance(&InstanceDesc {
            mesh: LANE_MESH + leg,
            material: LANE_MATERIAL,
            transform: at(middle.x, 0.0, middle.z),
        })?;
    }

    for plot in PLOTS {
        renderer.add_instance(&InstanceDesc {
            mesh: PAD_MESH,
            material: PAD_MATERIAL,
            transform: at(plot.x, 0.0, plot.z),
        })?;
    }

    let exit = PATH[PATH.len() - 1];
    renderer.add_instance(&InstanceDesc {
        mesh: EXIT_MESH,
        material: EXIT_MATERIAL,
        transform: at(exit.x, 0.0, exit.z),
    })?;

    // The pools last, every one of them parked: the first frame draws the field
    // before the first tick has spawned anything, and a pool left at the
    // origin would put a creep on the ground before the wave began.
    let mut creeps = Vec::with_capacity(MAX_CREEPS);
    for _ in 0..MAX_CREEPS {
        creeps.push(renderer.add_instance(&InstanceDesc {
            mesh: CREEP_MESH,
            material: CREEP_MATERIAL,
            transform: creep_transform(PARK),
        })?);
    }
    let mut towers = Vec::with_capacity(PLOTS.len());
    for _ in 0..PLOTS.len() {
        towers.push(renderer.add_instance(&InstanceDesc {
            mesh: TOWER_MESH,
            material: TOWER_MATERIAL,
            transform: creep_transform(PARK),
        })?);
    }
    let mut bolts = Vec::with_capacity(MAX_BOLTS);
    for _ in 0..MAX_BOLTS {
        bolts.push(renderer.add_instance(&InstanceDesc {
            mesh: BOLT_MESH,
            material: BOLT_MATERIAL,
            transform: creep_transform(PARK),
        })?);
    }

    // No point lights: this field is outdoors and [`sun`] is what lights it.
    renderer.set_lights(&[]);

    Ok(Field {
        creeps: creeps
            .try_into()
            .unwrap_or_else(|_| unreachable!("one instance per pooled creep was pushed")),
        towers: towers
            .try_into()
            .unwrap_or_else(|_| unreachable!("one instance per plot was pushed")),
        bolts: bolts
            .try_into()
            .unwrap_or_else(|_| unreachable!("one instance per pooled bolt was pushed")),
    })
}

// ---------------------------------------------------------------------------
// The collision side
// ---------------------------------------------------------------------------

/// The field as the colliders a bolt sweeps against, with the exit trigger's
/// id — which is what [`crate::creep`] compares an overlap's answer to.
///
/// Two colliders and no more: the ground, so a bolt whose target died stops in
/// it rather than flying under the map for ever, and the exit volume. The
/// creeps add their own — see the module docs.
#[must_use]
pub fn world() -> (PhysicsWorld, ColliderId) {
    let mut world = PhysicsWorld::new();
    world.add_box(BoxCollider::new(
        DVec3::new(0.0, -0.5 * SLAB_THICKNESS, 0.0),
        DVec3::new(HALF_WIDTH, 0.5 * SLAB_THICKNESS, HALF_DEPTH),
    ));
    let exit = world.add_box(exit_collider());
    world.set_trigger(exit, true);
    (world, exit)
}

// ---------------------------------------------------------------------------
// The light
// ---------------------------------------------------------------------------

/// How bright the sun is, before its colour.
const SUN_INTENSITY: f32 = 2.6;

/// How high it stands, as the `Y` component of a unit direction **toward** it.
const SUN_ELEVATION: f32 = 0.72;

/// The sun this field is lit by.
///
/// Fixed rather than turning: a tower defense is read from directly overhead
/// and a moving shadow would be the one thing on screen a player is not meant
/// to be watching. `apps/puppet::map::sun` is the one that turns, and it says
/// why it does.
#[must_use]
pub fn sun() -> DirectionalLight {
    let flat = (1.0 - SUN_ELEVATION * SUN_ELEVATION).sqrt();
    DirectionalLight {
        direction: Vec3::new(flat * -0.55, SUN_ELEVATION, flat * 0.84),
        color: Vec3::new(1.0, 0.97, 0.90) * SUN_INTENSITY,
        ambient: Vec3::new(0.17, 0.19, 0.22),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Every mesh the description makes resident is placed, and every row it
    /// declares is named**, in the order the constants say.
    ///
    /// A mesh nothing places is memory taken for geometry no frame draws, and a
    /// row nothing names is a colour nobody can see — both of which leave a
    /// perfectly plausible picture. `apps/breach/src/map.rs` and
    /// `apps/puppet/src/map.rs` assert the same pair.
    #[test]
    fn the_constants_name_their_own_meshes() {
        let scene = scene();
        assert_eq!(
            scene.meshes.len(),
            MESHES,
            "the mesh list is not MESHES long"
        );
        assert_eq!(
            scene.materials.len(),
            MATERIALS,
            "the material list is not MATERIALS long",
        );
        for (slot, label) in [
            (GROUND_MESH, "ground"),
            (LANE_MESH, "lane"),
            (PAD_MESH, "pad"),
            (EXIT_MESH, "exit"),
            (CREEP_MESH, "creep"),
            (TOWER_MESH, "tower"),
            (BOLT_MESH, "bolt"),
        ] {
            assert_eq!(
                scene.meshes[slot].label, label,
                "slot {slot} is not the {label}",
            );
        }
    }

    /// **The map fits the pools it reserves**, with the pooled creeps, towers
    /// and bolts counted in — the instances a run can reach are the fixed field
    /// plus every pool, and a description that only fitted an empty field would
    /// fail on the first wave rather than at start-up.
    #[test]
    fn the_map_fits_the_pools_it_reserves() {
        let placed = 1 + LEGS + PLOTS.len() + 1 + MAX_CREEPS + PLOTS.len() + MAX_BOLTS;
        assert!(
            placed <= CAPACITIES.instances as usize,
            "the map places {placed} instances into {}",
            CAPACITIES.instances,
        );
        assert!(MESHES <= CAPACITIES.meshes as usize);
        assert!(MATERIALS <= CAPACITIES.materials as usize);
    }

    /// **An unused instance is parked where the ground hides it**: under the
    /// slab, and inside its footprint. A park point beside the field would be a
    /// creep a player can see standing in the grass before the wave starts.
    #[test]
    fn the_parking_spot_is_under_the_ground() {
        // Every operand is a constant, so this is checked while the crate is
        // compiled rather than while its tests run — which is the earliest a
        // wrong number here could possibly be caught.
        const {
            assert!(
                PARK.y + CREEP_RADIUS < -SLAB_THICKNESS,
                "a parked creep pokes up through the ground",
            );
            assert!(
                PARK.x > -HALF_WIDTH && PARK.x < HALF_WIDTH,
                "the park point is off the field across X",
            );
            assert!(
                PARK.z > -HALF_DEPTH && PARK.z < HALF_DEPTH,
                "the park point is off the field along Z",
            );
        }
    }

    /// **Every leg of the path runs along one axis**, which is what lets the
    /// lane be one `platform` per leg and what [`crate::path`] measures.
    #[test]
    fn every_leg_of_the_path_is_axis_aligned() {
        for leg in 0..LEGS {
            let step = PATH[leg + 1] - PATH[leg];
            assert_eq!(step.y, 0.0, "leg {leg} climbs");
            assert!(
                (step.x == 0.0) != (step.z == 0.0),
                "leg {leg} runs diagonally, at {step:?}",
            );
            assert!(
                step.length() > LANE_WIDTH,
                "leg {leg} is shorter than the lane is wide"
            );
        }
    }

    /// **The whole path is on the field**, lane and all: a leg that ran off the
    /// slab would be creeps walking on nothing.
    #[test]
    fn the_path_stays_on_the_ground() {
        for point in PATH {
            assert!(
                point.x.abs() + 0.5 * LANE_WIDTH <= HALF_WIDTH,
                "{point:?} is off the field across X",
            );
            assert!(
                point.z.abs() + 0.5 * LANE_WIDTH <= HALF_DEPTH,
                "{point:?} is off the field along Z",
            );
        }
    }

    /// **The exit volume is a trigger, so a bolt goes through it and an overlap
    /// finds it.** The two halves of what `set_trigger` buys, asserted against
    /// the world this map builds rather than against the engine's own docs.
    ///
    /// The sweep is the control for the overlap: a build in which the exit was
    /// an ordinary box passes the overlap and fails the sweep, which is exactly
    /// the failure that would stop every bolt fired down the last leg.
    #[test]
    fn the_exit_is_a_volume_a_bolt_flies_through_and_an_overlap_reports() {
        use crcbl::phys::Segment;

        let (mut world, exit) = world();
        assert!(world.is_trigger(exit), "the exit was registered solid");

        let centre = exit_centre();
        assert!(
            world.overlap_sphere(centre, CREEP_RADIUS).contains(&exit),
            "a creep standing in the exit is not reported by the overlap",
        );

        // Straight through the volume, at the height a bolt aimed at a creep
        // standing in it would be.
        let across = Segment::new(
            centre + DVec3::new(4.0, 0.0, 0.0),
            centre + DVec3::new(-4.0, 0.0, 0.0),
        );
        assert_eq!(
            world.sweep_sphere(&across, BOLT_RADIUS),
            None,
            "the exit volume stopped a bolt",
        );
    }

    /// **Every plot stands clear of the lane**, so a tower is beside the path
    /// rather than on it, and **near enough to reach it**, so a plot is a build
    /// site rather than a decoration. The pair is what makes the five plots a
    /// choice.
    #[test]
    fn every_plot_stands_clear_of_the_lane_and_still_covers_it() {
        let clearance = 0.5 * LANE_WIDTH + TOWER_RADIUS;
        for plot in PLOTS {
            let mut nearest = f64::INFINITY;
            for leg in 0..LEGS {
                let (from, to) = (PATH[leg], PATH[leg + 1]);
                let step = to - from;
                let t = ((plot.at() - from).dot(step) / step.length_squared()).clamp(0.0, 1.0);
                nearest = nearest.min((from + step * t - plot.at()).length());
            }
            assert!(
                nearest > clearance,
                "{} sits {nearest:.2} m from the lane, inside the {clearance:.2} m it is wide",
                plot.label,
            );
            assert!(
                nearest < crate::tower::RANGE_M,
                "{} is {nearest:.2} m from the nearest leg, past the {} m a tower reaches",
                plot.label,
                crate::tower::RANGE_M,
            );
        }
    }
}
