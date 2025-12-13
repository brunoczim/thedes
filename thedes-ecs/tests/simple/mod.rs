use thedes_ecs::{
    value::{Entry, RawEntry},
    world::World,
};

#[test]
fn position_checks() -> anyhow::Result<()> {
    let mut world = World::new();
    let position = world.create_component();
    let speed = world.create_component();
    let acceleration = world.create_component();

    let ball = world.create_entity();
    world.create_value(ball, position)?;
    world.create_value(ball, speed)?;

    let player = world.create_entity();
    world.create_value(player, position)?;
    world.create_value(player, speed)?;
    world.create_value(player, acceleration)?;

    world.set_value(ball, position, 10.0_f64)?;
    world.set_value(ball, speed, 2.0_f64)?;
    world.set_value(player, position, 9.0_f64)?;
    world.set_value(player, speed, 1.0_f64)?;
    world.set_value(player, acceleration, 3.0_f64)?;

    world.create_system(
        [position, speed],
        |(mut position, speed): (Entry<f64>, Entry<f64>)| {
            position.set(position.get() + speed.get());
            Ok(())
        },
    );

    world.create_system_raw(
        [speed, acceleration],
        |entries: &mut [RawEntry]| {
            let speed: f64 = entries[0].get();
            let acceleration: f64 = entries[1].get();
            entries[0].set(speed * acceleration);
            Ok(())
        },
    );

    world.tick()?;
    world.tick()?;
    world.tick()?;
    world.tick()?;

    let ball_position: f64 = world.get_value(ball, position)?;
    let ball_speed: f64 = world.get_value(ball, speed)?;

    let player_position: f64 = world.get_value(player, position)?;
    let player_speed: f64 = world.get_value(player, speed)?;
    let player_acceleration: f64 = world.get_value(player, acceleration)?;

    println!("ball position: {ball_position}");
    println!("ball speed: {ball_speed}");
    println!("player position: {player_position}");
    println!("player speed: {player_speed}");
    println!("player acceleration: {player_acceleration}");

    assert_float_eq!(ball_speed, 2.0_f64);
    assert_float_eq!(ball_position, 18.0_f64);
    assert_float_eq!(player_speed, 81.0_f64);
    assert_float_eq!(player_position, 49.0_f64);
    assert_float_eq!(player_acceleration, 3.0_f64);

    Ok(())
}
