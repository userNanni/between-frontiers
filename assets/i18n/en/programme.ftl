# Nuclear programme: core plants and uranium ore (Forces tab)
programme-core-plants = Core plants
programme-none = none
programme-core-plant-value = { $levels } { $levels ->
    [one] level
   *[other] levels
}, { $cores } cores a year
programme-ore = Uranium ore a year
programme-ore-value = { $mined } kg from the country's mines; { $used } kg asked by the core plants
programme-ore-market = On the market: { $stock } kg in stock, { $price } a kg; the market delivered { $delivered } of what was asked.
# Warhead assembly: the assembly plant, the assemblies and the cores that wait
programme-assembly-plants = Assembly plants
programme-assembly-plant-value = { $levels } { $levels ->
    [one] level
   *[other] levels
}, { $made } assemblies a year
programme-assembly = Warhead assemblies on the market
programme-cores-waiting = Cores waiting for assembly
programme-cores-waiting-tip = Cores already bought for the warhead order that have no assembly yet (the explosive lenses, the initiator, the fuzing and the case). A warhead is ready only with both.
# Warhead order: what holds it back now and what it holds
programme-limit = What holds the warheads back now
programme-limit-money = the programme's money
programme-limit-cores = the core plants
programme-limit-assembly = the assembly plants
programme-limit-tip = The order grows until it spends all of the programme's money. When the market delivers less than nine tenths of the cores or assemblies bought, the plants of that good hold it back.
programme-order = Warhead order
programme-order-open = { $n } to make; warheads join one by one
programme-order-stopped = stopped: no programme
programme-order-held = In hand: { $cores } cores and { $assemblies } assemblies.
