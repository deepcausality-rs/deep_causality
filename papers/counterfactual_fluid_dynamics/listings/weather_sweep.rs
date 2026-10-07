let table = CfdFlow::study("weather-dispersion table")
    .cases(model::weather_cases())
    .baseline(model::standard_day)
    .alternate(model::weather_world)
    .ensemble(constants::MC_DRAWS)
    .couple(|case, draw| {
        trace::traced(world::corridor_coupling(
            model::bias_departure(case.d_temp),
            draw,
        ))
    })
    .march_for(constants::STEPS, world::initial_field)
    .reduce_ensemble(model::world_row)
    .gates(model::weather_gates())
    .verdict()?;
