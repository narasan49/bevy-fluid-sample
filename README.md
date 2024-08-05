# bevy fluid sample

This project is a fluid simulation sample built using the Bevy engine.

Try it on [here](https://narasan49.github.io/bevy-fluid-sample/)!

https://github.com/user-attachments/assets/2e7b5aba-e9ef-4fc7-9efa-7e71da4179f6

Screenshot: (Left) x-ward velocity field, (Right) pressure field with moving solid obstacles.

## Features
- [x] Invicid, incompressible 2D fluid simulation
- [x] Free boundary condition at the boundary of calculation domain
- [x] One-way solid body to fluid interaction

## Getting Started

To run the Bevy Fluid Sample, you will need to have Rust and Cargo installed on your system. Follow these steps to get up and running:

1. Clone the repository:
```ps1
git clone git@github.com:narasan49/bevy-fluid-sample.git
```

2. Navigate to the project directory:

```ps1
cd bevy-fluid-sample
```

3. Build and run the project:

```ps1
cargo run --example demo
```

## Acknowledgments
The simulation is inspired by and based on the algorithms described in these books:

- [Fluid Simulation for Computer Graphics](https://www.amazon.co.jp/dp/1482232839) by Robert Bridson
- [GPU Gems Chapter 38](https://developer.nvidia.com/gpugems/gpugems/part-vi-beyond-triangles/chapter-38-fast-fluid-dynamics-simulation-gpu) by Mark J. Harris
