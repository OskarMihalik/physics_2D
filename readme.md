# 2D physic engine demo 
This project contains physics engine written in Rust using Bevy game engine. The codebase is based on youtube video series by [Two-Bit Coding](https://www.youtube.com/playlist?list=PLSlpr6o9vURwq3oxVZSimY8iC-cdd3kIs).

# What the engine is doing
## Physics step
First the engine applyes linear and angular velocities and gravity to transform of a flat body. 

## Broad phase
In this step we detect if flat bodies are colliding using aabb collision. We save colliding objects for further processing

## Narrow phase
Now we iterate through saved colliding flat body pairs and find normal vector and depth. Normal vector tells us direction how to separate bodies and depth is how deep are the bodies inside each other. For this detection we are using separating axes theorem (SAT).
### SAT
image and explanation of how it works

### Separating bodies

### Finding contact points

### Resolving collision with rotation and friction

# Further possible improvements
- The bodies will start to wobble if stucked on each other
- Better integration with the engine
- Make it into a package for others to use
