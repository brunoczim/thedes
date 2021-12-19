use crate::map::Map;
use gardiz::{coord::Vec2, direc::Direction};
use thedes_common::{block::Block, human::Body, map::Coord, Result};

pub async fn write_on_map(
    body: Body,
    body_block: Block,
    map: &mut Map,
) -> Result<()> {
    map.entry_mut(body.head).await?.block = body_block;
    map.entry_mut(body.pointer()).await?.block = body_block;
    Ok(())
}

pub async fn erase_from_map(body: Body, map: &mut Map) -> Result<()> {
    write_on_map(body, Block::Empty, map).await
}

pub async fn move_around(
    body: &mut Body,
    body_block: Block,
    direc: Direction,
    map: &mut Map,
) -> Result<()> {
    if direc == body.facing {
        step(body, body_block, direc, map).await?;
    } else {
        turn_around(body, body_block, direc, map).await?;
    }

    Ok(())
}

pub async fn step(
    body: &mut Body,
    body_block: Block,
    direction: Direction,
    map: &mut Map,
) -> Result<()> {
    let maybe_head = body.head.checked_move(direction);
    let maybe_ptr = body.pointer().checked_move(direction);
    if let (Some(new_head), Some(new_ptr)) = (maybe_head, maybe_ptr) {
        if block_free(body_block, new_head, map).await?
            && block_free(body_block, new_ptr, map).await?
        {
            update_head(body, body_block, new_head, map).await?;
        }
    }
    Ok(())
}

pub async fn turn_around(
    body: &mut Body,
    body_block: Block,
    direc: Direction,
    map: &mut Map,
) -> Result<()> {
    let new_coord = match direc {
        Direction::Up => body
            .head
            .y
            .checked_sub(1)
            .map(|new_y| Vec2 { y: new_y, ..body.head }),

        Direction::Down => body
            .head
            .y
            .checked_add(1)
            .map(|new_y| Vec2 { y: new_y, ..body.head }),

        Direction::Left => body
            .head
            .x
            .checked_sub(1)
            .map(|new_x| Vec2 { x: new_x, ..body.head }),

        Direction::Right => body
            .head
            .x
            .checked_add(1)
            .map(|new_x| Vec2 { x: new_x, ..body.head }),
    };

    if let Some(new_coord) = new_coord {
        let empty = map.entry(new_coord).await?.block == Block::Empty;
        if empty {
            update_facing(body, body_block, direc, map).await?;
        }
    }

    Ok(())
}

pub async fn update_head(
    body: &mut Body,
    body_block: Block,
    pos: Vec2<Coord>,
    map: &mut Map,
) -> Result<()> {
    erase_from_map(*body, map).await?;
    body.head = pos;
    write_on_map(*body, body_block, map).await?;
    Ok(())
}

/// Updates the facing direction and the map blocks too.
pub async fn update_facing(
    body: &mut Body,
    body_block: Block,
    direc: Direction,
    map: &mut Map,
) -> Result<()> {
    erase_from_map(*body, map).await?;
    body.facing = direc;
    write_on_map(*body, body_block, map).await?;
    Ok(())
}

pub async fn block_free(
    body_block: Block,
    pos: Vec2<Coord>,
    map: &mut Map,
) -> Result<bool> {
    let block = map.entry(pos).await?.block;
    Ok(block == Block::Empty || block == body_block)
}
