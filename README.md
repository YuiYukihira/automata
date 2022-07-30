Automata
========

This is a little toy I made with [bevy](https://github.com/bevyengine/bevy) just to experiment with it.


## Running
Just clone the repo ala:

`git clone git@github.com:YuiYukihira/automata.git`

And then run it with cargo specifying the preset in the assets directory:

`cargo run -- glider.rle`

## Controls
The spacebar pauses and plays the simulation.
The arrow and wasd keys move the camera.

## World size and Performance
Permformance isn't the greatest but the world is as big a a `i32` can store, so
go ham. I've at least made it so computations are only done on alive tiles and their neighbours,
so you should see you PC slow down respective to that, not the size of the board.

## Presets
The program supports [lifewiki](https://conwaylife.com/wiki/Main_Page) standard files: `.rle` and `.cells`. And another basic one `.board` with you can see the format of in `acorn.board`.

To add another preset, bung it in the assets directory and it should be usable.

## Plans
I'm planning to support other rulesets that the classic B2/S23 (Conway's game of life).
The code is structured for it, I just need to write them in.

I also want to dynamically set the ruleset from `.rle` files if they set it.

## Known issues
The simulation isn't exactly consistent right now. You can see this easily with
`gosper.board`, sometimes it will fizzle out instead of contantly producing gliders.

## Contributing
You see something wrong? Have I done something stupid? Feel free to raise a PR and explain what you're trying to accomplish.

# License
<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br/>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>