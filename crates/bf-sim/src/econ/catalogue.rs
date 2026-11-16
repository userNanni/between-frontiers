//! The economy catalogue indexed for the simulation: keys become dense indices and every
//! quantity becomes [`Fixed`]. Built once from the data files; never saved (it is game data).

use laplace_schelling::Fixed;

use crate::economy::{BuildingsFile, GoodsFile, NeedsFile, PROFESSIONS};
use crate::resources::{FARMS, RESOURCES};

/// Weeks in the year the catalogue's annual quantities are divided by.
pub const WEEKS: i64 = 52;

pub type GoodIx = u16;

#[derive(Clone, Debug)]
pub struct Good {
    pub key: String,
    pub category: String,
    pub base_price: Fixed,
    pub available_from: i32,
    pub tradable: bool,
}

#[derive(Clone, Debug)]
pub struct Method {
    pub key: String,
    pub available_from: i32,
    /// Per level per week.
    pub inputs: Vec<(GoodIx, Fixed)>,
    pub outputs: Vec<(GoodIx, Fixed)>,
    /// Workers per level, by profession index.
    pub jobs: Vec<(u8, i64)>,
    pub workers: i64,
}

#[derive(Clone, Debug)]
pub struct Building {
    pub key: String,
    pub group: String,
    /// Index into `economy::BUILDING_GROUPS`.
    pub group_ix: u8,
    /// Index into `RESOURCES`.
    pub resource: Option<u8>,
    /// Index into `FARMS`.
    pub farm: Option<u8>,
    /// Services gain productivity slowly (a barber cuts as much hair in 2025 as in 1925), so
    /// labour moves to them as the rest of the economy grows.
    pub stagnant: bool,
    pub methods: Vec<Method>,
}

#[derive(Clone, Debug)]
pub struct Need {
    pub key: String,
    pub tier: u8,
    /// Units per million people per week (a person's weekly share is too small for `Fixed`).
    pub per_capita: Fixed,
    pub goods: Vec<(GoodIx, Fixed)>,
}

#[derive(Clone, Debug)]
pub struct Catalogue {
    pub goods: Vec<Good>,
    pub buildings: Vec<Building>,
    pub needs: Vec<Need>,
}

#[derive(Debug)]
pub struct CatalogueError(pub String);

impl std::fmt::Display for CatalogueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Data values are decimal text: going through `f64` once at load is exact enough and the same
/// on every machine, since parsing is correctly rounded.
fn fx(v: f32) -> Fixed {
    Fixed::from_f64_lossy(f64::from(v))
}

impl Catalogue {
    pub fn build(
        goods: &GoodsFile,
        buildings: &BuildingsFile,
        needs: &NeedsFile,
    ) -> Result<Self, CatalogueError> {
        let gx = |k: &str| -> Result<GoodIx, CatalogueError> {
            goods
                .goods
                .iter()
                .position(|g| g.key == k)
                .map(|i| i as GoodIx)
                .ok_or_else(|| CatalogueError(format!("unknown good {k}")))
        };
        let per_week = |q: f32| fx(q).mul_ratio(1, WEEKS);
        let mut bs = Vec::new();
        for b in &buildings.buildings {
            let mut methods = Vec::new();
            for m in &b.methods {
                let jobs: Vec<(u8, i64)> = m
                    .jobs
                    .iter()
                    .map(|(p, n)| {
                        PROFESSIONS
                            .iter()
                            .position(|x| x == p)
                            .map(|i| (i as u8, i64::from(*n)))
                            .ok_or_else(|| CatalogueError(format!("unknown profession {p}")))
                    })
                    .collect::<Result<_, _>>()?;
                methods.push(Method {
                    key: m.key.clone(),
                    available_from: m.available_from,
                    inputs: m
                        .inputs
                        .iter()
                        .map(|(g, q)| Ok((gx(g)?, per_week(*q))))
                        .collect::<Result<_, CatalogueError>>()?,
                    outputs: m
                        .outputs
                        .iter()
                        .map(|(g, q)| Ok((gx(g)?, per_week(*q))))
                        .collect::<Result<_, CatalogueError>>()?,
                    workers: jobs.iter().map(|(_, n)| n).sum(),
                    jobs,
                });
            }
            bs.push(Building {
                key: b.key.clone(),
                group: b.group.clone(),
                group_ix: crate::economy::BUILDING_GROUPS
                    .iter()
                    .position(|g| *g == b.group)
                    .ok_or_else(|| CatalogueError(format!("unknown group {}", b.group)))?
                    as u8,
                stagnant: b.group == "services",
                resource: b
                    .resource
                    .as_ref()
                    .and_then(|r| RESOURCES.iter().position(|x| x == r))
                    .map(|i| i as u8),
                farm: b
                    .farm
                    .as_ref()
                    .and_then(|f| FARMS.iter().position(|x| x == f))
                    .map(|i| i as u8),
                methods,
            });
        }
        let ns = needs
            .needs
            .iter()
            .map(|n| {
                Ok(Need {
                    key: n.key.clone(),
                    tier: n.tier,
                    per_capita: fx(n.per_capita).mul_ratio(1_000_000, WEEKS),
                    goods: n
                        .goods
                        .iter()
                        .map(|(g, w)| Ok((gx(g)?, fx(*w))))
                        .collect::<Result<_, CatalogueError>>()?,
                })
            })
            .collect::<Result<_, CatalogueError>>()?;
        Ok(Self {
            goods: goods
                .goods
                .iter()
                .map(|g| Good {
                    key: g.key.clone(),
                    category: g.category.clone(),
                    base_price: Fixed::from_int(i64::from(g.base_price)),
                    available_from: g.available_from,
                    tradable: g.tradable,
                })
                .collect(),
            buildings: bs,
            needs: ns,
        })
    }

    pub fn good(&self, key: &str) -> Option<GoodIx> {
        self.goods
            .iter()
            .position(|g| g.key == key)
            .map(|i| i as GoodIx)
    }

    pub fn building(&self, key: &str) -> Option<usize> {
        self.buildings.iter().position(|b| b.key == key)
    }

    /// Best method index available in `year` by the given ranking, or the first one.
    pub fn first_method(&self, building: usize, year: i32) -> Option<usize> {
        self.buildings[building]
            .methods
            .iter()
            .position(|m| m.available_from <= year)
    }
}
