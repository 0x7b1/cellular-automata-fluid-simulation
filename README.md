# Cellular Automata Fluid Simulation

This is an implementation of fluid/liquid dynamics using Cellular Automata.
The purpose of this project is to show how dramatic is the difference of computing the evolution of cells with two approaches:
naive grid cell evolution in CPU, and compute shaders in GPU.

You can find more about the journey of this project [here](https://courses.cs.ut.ee/2020/cg-pro/spring/Main/Project-AutomataSandbox).

![Screenshot](benches/gpu_sim_1.gif)

## How to play

- The elements that can be drawn on the canvas are:
  - Water <kbd>NumKey 1</kbd>
  - Ground <kbd>NumKey 2</kbd>
  - Acid <kbd>NumKey 3</kbd>
- Increase the size of the brush with the mouse wheel 
- Generate a new procedural cave map with <kbd>N</kbd>
- Clean the map with <kbd>C</kbd>
- Rotate the map with <kbd>R</kbd>

## Requirements

- Rust
- GLFW

## How to run

- Clone the repository

```shell script
$ git clone https://github.com/0x7b1/cellular-automata-fluid-simulation.git
$ cd cellular-automata-fluid-simulation
```

- Run the simulation

```rust
$ cargo run
```

### Resources

- http://www.jgallant.com/2d-liquid-simulator-with-cellular-automaton-in-unity/
- https://jonathansteyfkens.com/blog/rust/2018/08/07/rust-conway-game-of-life.html
- https://maxbittker.com/making-sandspiel
- https://thebookofshaders.com/edit.php?log=160909064723
- https://thebookofshaders.com/edit.php?log=160909064528
- https://thebookofshaders.com/edit.php?log=161127202429
- https://nullprogram.com/blog/2020/04/30/
