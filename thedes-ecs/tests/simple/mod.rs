use thedes_ecs::world::World;

#[test]
fn position_checks() -> anyhow::Result<()> {
    let mut world = World::new();
    let position = world.create_component::<f64>();
    let speed = world.create_component::<f64>();
    let acceleration = world.create_component::<f64>();

    let ball = world.create_entity();
    world.create_value(ball, position, 10.0)?;
    world.create_value(ball, speed, 2.0)?;

    let player = world.create_entity();
    world.create_value(player, position, 9.0)?;
    world.create_value(player, speed, 1.0)?;
    world.create_value(player, acceleration, 3.0)?;

    world.create_system((position, speed), |(mut position, speed)| {
        position.set(position.get() + speed.get());
        Ok(())
    });

    world.create_system((speed, acceleration), |(mut speed, acceleration)| {
        speed.set(speed.get() * acceleration.get());
        Ok(())
    });

    world.tick()?;
    world.tick()?;
    world.tick()?;
    world.tick()?;

    let ball_position = world.get_value(ball, position)?;
    let ball_speed = world.get_value(ball, speed)?;

    let player_position = world.get_value(player, position)?;
    let player_speed = world.get_value(player, speed)?;
    let player_acceleration = world.get_value(player, acceleration)?;

    println!("ball position: {ball_position}");
    println!("ball speed: {ball_speed}");
    println!("player position: {player_position}");
    println!("player speed: {player_speed}");
    println!("player acceleration: {player_acceleration}");

    assert_float_eq!(ball_speed, 2.0);
    assert_float_eq!(ball_position, 18.0);
    assert_float_eq!(player_speed, 81.0);
    assert_float_eq!(player_position, 49.0);
    assert_float_eq!(player_acceleration, 3.0);

    Ok(())
}
