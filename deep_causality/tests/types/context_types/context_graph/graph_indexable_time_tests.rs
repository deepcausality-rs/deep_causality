/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::*;

fn get_context() -> BaseContext {
    let id = 1;
    let name = "base context";
    Context::with_capacity(id, name, 10)
}

#[test]
fn test_set_get_current_time_index_direct() {
    let mut ctx = get_context();

    ctx.set_time_index(TimeScale::Hour as usize, 10, true);
    assert_eq!(
        ctx.get_time_index(&(TimeScale::Hour as usize), true),
        Some(&10)
    );

    ctx.set_time_index(TimeScale::Day as usize, 42, true);
    assert_eq!(
        ctx.get_time_index(&(TimeScale::Day as usize), true),
        Some(&42)
    );
}

#[test]
fn test_set_get_previous_time_index_direct() {
    let mut ctx = get_context();

    ctx.set_time_index(TimeScale::Month as usize, 5, false);
    assert_eq!(
        ctx.get_time_index(&(TimeScale::Month as usize), false),
        Some(&5)
    );

    ctx.set_time_index(TimeScale::Year as usize, 2024, false);
    assert_eq!(
        ctx.get_time_index(&(TimeScale::Year as usize), false),
        Some(&2024)
    );
}

#[test]
fn test_set_get_current_time_index_trait() {
    let mut ctx = get_context();

    ctx.set_current_minute_index(59);
    assert_eq!(ctx.get_current_minute_index(), Some(&59));

    ctx.set_current_hour_index(23);
    assert_eq!(ctx.get_current_hour_index(), Some(&23));

    ctx.set_current_day_index(15);
    assert_eq!(ctx.get_current_day_index(), Some(&15));

    ctx.set_current_week_index(2);
    assert_eq!(ctx.get_current_week_index(), Some(&2));

    ctx.set_current_month_index(6);
    assert_eq!(ctx.get_current_month_index(), Some(&6));

    ctx.set_current_year_index(2025);
    assert_eq!(ctx.get_current_year_index(), Some(&2025));
}

#[test]
fn test_set_get_previous_time_index_trait() {
    let mut ctx = get_context();

    ctx.set_previous_minute_index(58);
    assert_eq!(ctx.get_previous_minute_index(), Some(&58));

    ctx.set_previous_hour_index(22);
    assert_eq!(ctx.get_previous_hour_index(), Some(&22));

    ctx.set_previous_day_index(14);
    assert_eq!(ctx.get_previous_day_index(), Some(&14));

    ctx.set_previous_week_index(1);
    assert_eq!(ctx.get_previous_week_index(), Some(&1));

    ctx.set_previous_month_index(5);
    assert_eq!(ctx.get_previous_month_index(), Some(&5));

    ctx.set_previous_year_index(2024);
    assert_eq!(ctx.get_previous_year_index(), Some(&2024));
}

#[test]
fn test_missing_index_returns_none() {
    let ctx = get_context();
    assert_eq!(ctx.get_current_minute_index(), None);
    assert_eq!(ctx.get_previous_hour_index(), None);
}
