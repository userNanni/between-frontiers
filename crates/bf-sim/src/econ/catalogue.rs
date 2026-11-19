//! The economy catalogue indexed for the simulation: keys become dense indices and every
//! quantity becomes [`Fixed`]. Built once from the data files; never saved (it is game data).

use laplace_schelling::Fixed;

use crate::economy::{BuildingsFile, GoodsFile, NeedsFile, PROFESSIONS, PurchasesFile};
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
    /// Clears state by state (`GoodDef::local`).
    pub local: bool,
    /// Technology key that makes it flow across the country (`GoodDef::national_with`).
    pub national_with: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct Method {
    pub key: String,
    pub available_from: i32,
    /// Per level per week.
    pub inputs: Vec<(GoodIx, Fixed)>,
    pub outputs: Vec<(GoodIx, Fixed)>,
    /// Workers per level, by profession index.
    pub jobs: Vec<(u8, i64)>,
    pub workers: i64,
    /// Transport the output needs to reach its buyers, per level per week, already among the
    /// inputs: a plant without it in its state reaches the market only in part.
    pub freight: Fixed,
    /// Value of the outputs of a level at base prices.
    pub output_value: Fixed,
    /// The jobs weighted by `WAGE_WEIGHT`: the wage bill is shared in these proportions.
    pub job_weight: Fixed,
    /// Capital of a level relative to a level of the building's first method, when the
    /// building declares it (`MethodDef::capital`): the choice of method weighs the return on
    /// it, and building or switching to the method pays it.
    pub capital: Option<Fixed>,
}

impl Method {
    /// Outputs less inputs of a level, each good valued at `price`.
    pub fn value_added(&self, price: impl Fn(usize) -> Fixed) -> Fixed {
        let value = |items: &[(GoodIx, Fixed)]| -> Fixed {
            items.iter().map(|(g, q)| *q * price(*g as usize)).sum()
        };
        value(&self.outputs) - value(&self.inputs)
    }

    /// Weekly profit of a level at `price`: value added × productivity, less wages.
    pub fn profit(&self, price: &[Fixed], tfp: Fixed, wage: Fixed) -> Fixed {
        self.value_added(|g| price[g]) * tfp - wage * self.workers
    }
}

#[derive(Clone, Debug, Default)]
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
    /// Built and paid for by the state (`BuildingDef::public`).
    pub public: bool,
    /// Its profit goes whole to the treasury (`BuildingDef::state_firm`).
    pub state_firm: bool,
    /// Urban self-employment sized by the labour market (`BuildingDef::informal`).
    pub informal: bool,
    /// Goods of one new level, per mille of its cost, and whether each is the building itself
    /// (bought from the building trade as construction points) ([`PurchasesFile`]).
    pub construction: Vec<(GoodIx, i64, bool)>,
    /// Its profits go to landowners rather than capitalists.
    pub agriculture: bool,
    /// Share of value added paid as wages ([`super::labor_share`]).
    pub labor_share: Fixed,
    pub methods: Vec<Method>,
}

#[derive(Clone, Debug)]
pub struct Need {
    pub key: String,
    pub tier: u8,
    /// Units per million people per week (a person's weekly share is too small for `Fixed`).
    pub per_capita: Fixed,
    /// Who has the need: urban professions, rural ones, or everyone (`None`).
    pub urban: Option<bool>,
    pub goods: Vec<(GoodIx, Fixed)>,
    /// Weight of the need in the money left once every tier is met (`NeedDef::spare`).
    pub spare_weight: Fixed,
}

#[derive(Clone, Debug, Default)]
pub struct Catalogue {
    pub goods: Vec<Good>,
    /// Freight of each good over the reference distance of an international haul
    /// (`TradeDef::reference_km`), as a share of its value: its producers' group freight
    /// (`purchases.ron`).
    pub good_freight: Vec<Fixed>,
    /// Building group of the first building that makes each good: trading networks and the
    /// preferences of trade are by group (`econ::trade`). `None` for goods nobody makes.
    pub good_group: Vec<Option<u8>>,
    /// Trade with a partner (`PurchasesFile::trade`).
    pub trade: crate::economy::TradeDef,
    pub buildings: Vec<Building>,
    pub needs: Vec<Need>,
    /// Goods that clear state by state, in catalogue order.
    pub local: Vec<GoodIx>,
    /// The good freight is paid in (transport).
    pub freight: Option<GoodIx>,
    /// Goods and needs the rules name, looked up once instead of by key every week.
    pub grain: Option<GoodIx>,
    pub construction: Option<GoodIx>,
    pub staple_need: Option<usize>,
    /// Farms that grow a staple, as (farm index, building, staples a level of its first method
    /// grows in a week): the land smallholders can till when the farms leave it free.
    pub staple_farms: Vec<(u8, u16, Fixed)>,
    /// Land a level of each farm (`FARMS` index) works, km² (`resources::land_per_level`).
    pub farm_km2: Vec<i64>,
    /// The ranch and the forestry farms (`FARMS` index), whose land clearing takes.
    pub pasture_farm: Option<u8>,
    pub forest_farm: Option<u8>,
    /// The urban self-employment building, if the catalogue has one.
    pub informal: Option<u16>,
    /// Goods the state buys whole (category `public`: schools, hospitals, research).
    pub public_goods: Vec<GoodIx>,
    /// Per good: of the `military` category (arms the forces order); resolved at load so the
    /// tick does not compare names.
    pub military: Vec<bool>,
    /// Research and development, and what each building group's firms want of it, as a share
    /// of the value the group adds (`economy::BUILDING_GROUPS` order).
    pub research: Option<GoodIx>,
    pub research_intensity: Vec<Fixed>,
    /// The building that makes research.
    pub laboratory: Option<u16>,
    /// What the state and the investment pool buy, per mille ([`PurchasesFile`]).
    pub state_basket: Vec<(GoodIx, i64)>,
    pub upkeep_basket: Vec<(GoodIx, i64)>,
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
        purchases: &PurchasesFile,
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
        let basket = |items: &[(String, i64)]| -> Result<Vec<(GoodIx, i64)>, CatalogueError> {
            items
                .iter()
                .map(|(k, share)| Ok((gx(k)?, *share)))
                .collect()
        };
        let group_of = |key: &str| {
            purchases
                .groups
                .iter()
                .find(|g| g.group == key)
                .ok_or_else(|| CatalogueError(format!("no purchases for building group {key}")))
        };
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
                    job_weight: jobs
                        .iter()
                        .map(|(p, n)| super::WAGE_WEIGHT[*p as usize] * *n)
                        .sum(),
                    jobs,
                    freight: Fixed::ZERO,
                    output_value: Fixed::ZERO,
                    capital: m.capital.map(fx),
                });
            }
            // Capital is declared for every method of a building or for none, relative to the
            // first: a choice between a priced method and an unpriced one would mean nothing.
            let declared = methods.iter().filter(|m| m.capital.is_some()).count();
            if declared != 0 && declared != methods.len() {
                return Err(CatalogueError(format!(
                    "{}: capital declared for {declared} of {} methods",
                    b.key,
                    methods.len()
                )));
            }
            if let Some(first) = methods.first().and_then(|m| m.capital)
                && first != Fixed::ONE
            {
                return Err(CatalogueError(format!(
                    "{}: the first method's capital is the unit, 1",
                    b.key
                )));
            }
            if methods
                .iter()
                .any(|m| m.capital.is_some_and(|c| c <= Fixed::ZERO))
            {
                return Err(CatalogueError(format!(
                    "{}: capital must be positive",
                    b.key
                )));
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
                public: b.public,
                state_firm: b.state_firm,
                informal: b.informal,
                agriculture: b.group == "agriculture",
                labor_share: super::labor_share(&b.group),
                construction: group_of(&b.group)?
                    .construction
                    .iter()
                    .map(|(k, share)| Ok((gx(k)?, *share, purchases.building_work.contains(k))))
                    .collect::<Result<_, CatalogueError>>()?,
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
        let ns: Vec<Need> = needs
            .needs
            .iter()
            .map(|n| {
                if n.tier >= 3 && n.spare <= 0.0 {
                    return Err(CatalogueError(format!(
                        "need {} has no spare weight",
                        n.key
                    )));
                }
                Ok(Need {
                    key: n.key.clone(),
                    tier: n.tier,
                    per_capita: fx(n.per_capita).mul_ratio(1_000_000, WEEKS),
                    urban: n.urban,
                    spare_weight: fx(n.spare),
                    goods: n
                        .goods
                        .iter()
                        .map(|(g, w)| Ok((gx(g)?, fx(*w))))
                        .collect::<Result<_, CatalogueError>>()?,
                })
            })
            .collect::<Result<_, CatalogueError>>()?;
        let goods: Vec<Good> = goods
            .goods
            .iter()
            .map(|g| Good {
                key: g.key.clone(),
                category: g.category.clone(),
                base_price: Fixed::from_int(i64::from(g.base_price)),
                available_from: g.available_from,
                tradable: g.tradable,
                local: g.local,
                national_with: g.national_with.clone(),
            })
            .collect();
        let freight = gx("transportation").ok();
        // Freight joins the inputs of the plants that ship goods.
        if let Some(fg) = freight {
            let fprice = goods[fg as usize].base_price;
            for b in &mut bs {
                let per_mille = group_of(&b.group)?.freight;
                for m in &mut b.methods {
                    let shipped: Fixed = m
                        .outputs
                        .iter()
                        .filter(|(g, _)| goods[*g as usize].tradable)
                        .map(|(g, q)| *q * goods[*g as usize].base_price)
                        .sum();
                    let q = shipped.mul_ratio(per_mille, 1000) / fprice;
                    if q <= Fixed::ZERO {
                        continue;
                    }
                    m.freight = q;
                    match m.inputs.iter_mut().find(|(g, _)| *g == fg) {
                        Some(e) => e.1 += q,
                        None => m.inputs.push((fg, q)),
                    }
                }
            }
        }
        for m in bs.iter_mut().flat_map(|b| b.methods.iter_mut()) {
            m.output_value = m
                .outputs
                .iter()
                .map(|(g, q)| *q * goods[*g as usize].base_price)
                .sum();
        }
        let goods_where = |keep: &dyn Fn(&Good) -> bool| -> Vec<GoodIx> {
            (0..goods.len())
                .filter(|&g| keep(&goods[g]))
                .map(|g| g as GoodIx)
                .collect()
        };
        let staple_need = ns.iter().position(|n| n.key == "staple_food");
        let staples: Vec<GoodIx> = staple_need
            .map(|i| ns[i].goods.iter().map(|(g, _)| *g).collect())
            .unwrap_or_default();
        let staple_farms = bs
            .iter()
            .enumerate()
            .filter_map(|(k, b)| {
                let farm = b.farm?;
                let q: Fixed = b
                    .methods
                    .first()?
                    .outputs
                    .iter()
                    .filter(|(g, _)| staples.contains(g))
                    .map(|(_, q)| *q)
                    .sum();
                (q > Fixed::ZERO).then_some((farm, k as u16, q))
            })
            .collect();
        let informal = bs.iter().position(|b| b.informal).map(|k| k as u16);
        Ok(Self {
            farm_km2: FARMS
                .iter()
                .map(|f| crate::resources::land_per_level(f))
                .collect(),
            pasture_farm: FARMS
                .iter()
                .position(|f| *f == "livestock_ranch")
                .map(|i| i as u8),
            forest_farm: FARMS.iter().position(|f| *f == "forestry").map(|i| i as u8),
            informal,
            staple_farms,
            grain: gx("grain").ok(),
            construction: gx("construction").ok(),
            staple_need,
            local: goods_where(&|g| g.local),
            public_goods: goods_where(&|g| g.category == "public"),
            military: goods.iter().map(|g| g.category == "military").collect(),
            research: gx("research").ok(),
            laboratory: bs
                .iter()
                .position(|b: &Building| b.key == "laboratory")
                .map(|i| i as u16),
            research_intensity: crate::economy::BUILDING_GROUPS
                .iter()
                .map(|k| group_of(k).map(|g| Fixed::from_ratio(g.research, 1000)))
                .collect::<Result<_, _>>()?,
            state_basket: basket(&purchases.state)?,
            upkeep_basket: basket(&purchases.upkeep)?,
            good_group: {
                let mut group = vec![None; goods.len()];
                for b in &bs {
                    for m in &b.methods {
                        for (g, _) in &m.outputs {
                            group[*g as usize].get_or_insert(b.group_ix);
                        }
                    }
                }
                group
            },
            good_freight: {
                // The highest freight among the groups whose buildings make the good.
                let mut per_mille = vec![0i64; goods.len()];
                for b in &bs {
                    let f = group_of(&b.group)?.freight;
                    for m in &b.methods {
                        for (g, _) in &m.outputs {
                            let e = &mut per_mille[*g as usize];
                            *e = (*e).max(f);
                        }
                    }
                }
                per_mille
                    .into_iter()
                    .map(|p| Fixed::from_ratio(p, 1000))
                    .collect()
            },
            trade: purchases.trade.clone(),
            goods,
            buildings: bs,
            needs: ns,
            freight,
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
