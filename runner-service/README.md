# Runner Service

This is the "code execution node" component. A runner receives a job from a controller node, compiles / executes the code in a isolated container, and throws the result back at the controller. Breaking out the runner into a seperate service allows us to horizontally autoscale it for varying loads vs vertically scaling a single monolithic app.