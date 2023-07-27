// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use std::error::Error;
use chrono::Datelike;
use deep_causality::prelude::{ContextMatrixGraph, Node, NodeType, RelationKind, Root, Spaceoid, SpaceTempoid, Tempoid, TimeScale};
use crate::types::counter;
use crate::types::dateoid::Dataoid;
use crate::types::sampled_date_time_bar::SampledDataBars;
use crate::workflow::augment_data;

const NODE_CAPACITY: usize = 525600;

pub fn generate_time_data_context_graph(
    data: &SampledDataBars,
    time_scale: TimeScale,
)
    -> Result<ContextMatrixGraph<Dataoid, Spaceoid, Tempoid, SpaceTempoid>, Box<dyn Error>>
{
    let counter = counter::RelaxedAtomicCounter::new();

    let mut g = ContextMatrixGraph::with_capacity(NODE_CAPACITY);

    let cm = get_boolean_control_map(time_scale);
    let add_month = *cm.get(2).unwrap();
    let add_week = *cm.get(3).unwrap();
    let add_day = *cm.get(4).unwrap();

    // == ADD ROOT ==//
    let id = counter.increment_and_get();
    let root = Root::new(id);
    let root_node = Node::new(id, NodeType::Root(root));
    let root_index = g.add_node(root_node);

    // == ADD YEAR ==//
    let time_scale = TimeScale::Year;
    let elements = data.year_bars();
    for data_bar in elements {
        let year = data_bar.date_time().year();

        let (tempoid, dataoid) = augment_data::convert_bar_to_augmented(data_bar, time_scale);

        let key = counter.increment_and_get();
        let time_node = Node::new(key, NodeType::Tempoid(tempoid));
        let year_index = g.add_node(time_node);

        let data_id = counter.increment_and_get();
        let data_node = Node::new(data_id, NodeType::Datoid(dataoid));
        let data_index = g.add_node(data_node);

        // link root to year
        g.add_edge(
            root_index,
            year_index,
            RelationKind::Temporal,
        );

        // link data to year
        g.add_edge(
            data_index,
            year_index,
            RelationKind::Datial,
        );

        if !add_month {
            continue;
        }

        // == ADD MONTH FOR EACH YEAR ==//
        let time_scale = TimeScale::Month;
        let elements = data.month_bars();
        for data_bar in elements {
            let month = data_bar.date_time().month();

            if data_bar.date_time().year() != year {
                continue;
            }

            let (tempoid, dataoid) = augment_data::convert_bar_to_augmented(data_bar, time_scale);

            // Add Month
            let key = counter.increment_and_get();
            let time_node = Node::new(key, NodeType::Tempoid(tempoid));
            let month_index = g.add_node(time_node);

            // Add data
            let data_id = counter.increment_and_get();
            let data_node = Node::new(data_id, NodeType::Datoid(dataoid));
            let data_index = g.add_node(data_node);

            // link month to year
            g.add_edge(
                month_index,
                year_index,
                RelationKind::Temporal,
            );

            // link data to month
            g.add_edge(
                data_index,
                month_index,
                RelationKind::Datial,
            );

            if !add_week {
                continue;
            }

            // == ADD WEEK FOR EACH MONTH ==//
            let time_scale = TimeScale::Week;
            let elements = data.week_bars();
            for data_bar in elements {
                let week = data_bar.date_time().iso_week().week();

                if data_bar.date_time().year() != year {
                    continue;
                }

                if data_bar.date_time().month() != month {
                    continue;
                }

                let (tempoid, dataoid) = augment_data::convert_bar_to_augmented(data_bar, time_scale);

                // Add Week
                let key = counter.increment_and_get();
                let time_node = Node::new(key, NodeType::Tempoid(tempoid));
                let week_index = g.add_node(time_node);

                // Add data
                let data_id = counter.increment_and_get();
                let data_node = Node::new(data_id, NodeType::Datoid(dataoid));
                let data_index = g.add_node(data_node);

                // link week to month
                g.add_edge(
                    week_index,
                    month_index,
                    RelationKind::Temporal,
                );

                // link data to week
                g.add_edge(
                    data_index,
                    week_index,
                    RelationKind::Datial,
                );

                if !add_day {
                    continue;
                }

                // == ADD DAY FOR EACH WEEK ==//
                let time_scale = TimeScale::Day;
                let elements = data.day_bars();
                for data_bar in elements {
                    if data_bar.date_time().year() != year {
                        continue;
                    }

                    if data_bar.date_time().month() != month {
                        continue;
                    }

                    if data_bar.date_time().iso_week().week() != week {
                        continue;
                    }

                    let (tempoid, dataoid) = augment_data::convert_bar_to_augmented(data_bar, time_scale);

                    // Add day
                    let key = counter.increment_and_get();
                    let time_node = Node::new(key, NodeType::Tempoid(tempoid));
                    let day_index = g.add_node(time_node);

                    // Add data
                    let data_id = counter.increment_and_get();
                    let data_node = Node::new(data_id, NodeType::Datoid(dataoid));
                    let data_index = g.add_node(data_node);

                    // link day to week
                    g.add_edge(
                        day_index,
                        week_index,
                        RelationKind::Temporal,
                    );

                    // link data to week
                    g.add_edge(
                        data_index,
                        day_index,
                        RelationKind::Datial,
                    );
                } // end day
            } // end week
        } // end month
    } // end year

    Ok(g)
}

fn get_boolean_control_map(
    time_scale: TimeScale
)
    -> Vec<bool>
{
    match time_scale {
        // Boolean Index:
        // 0: Year,1: Quarter,2: Month,3: Week,4: Day,5: Hour,6: Minute, 7: Second
        TimeScale::NoScale => vec![true, true, true, true, true, true, true, true],
        TimeScale::Second => vec![true, true, true, true, true, true, true, true],
        TimeScale::Minute => vec![true, true, true, true, true, true, true, false],
        TimeScale::Hour => vec![true, true, true, true, true, true, false, false],
        TimeScale::Day => vec![true, true, true, true, true, false, false, false],
        TimeScale::Week => vec![true, true, true, true, false, false, false, false],
        TimeScale::Month => vec![true, true, true, false, false, false, false, false],
        TimeScale::Quarter => vec![true, true, false, false, false, false, false, false],
        TimeScale::Year => vec![true, false, false, false, false, false, false, false],
    }
}