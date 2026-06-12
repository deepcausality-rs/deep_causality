## The cfd.univeristy community challenge

I want you to beat me! Not physically, I mean, I spend most of my time in front of my laptop, typing on my Cherry G80 with my patented 2.5 finger typing style (I like to think that I can use all 10 fingers for typing, but my wife disagrees, so we meet in the middle at 2.5 ...) and I drink way too much royal milk tea at the moment. I am probably stage-2 lactose intolerant by now, but I just can't stop. I have no means to defend myself.

No, instead, I want you to beat my Navier-Stokes solver. I want you to beat it on two metrics: I want your solver to be **faster** than mine, and I want your solver to be **more accurate** than mine. The catch: you have to use a large language model (LLM, e.g. ChatGPT, Claude, etc.) to write the code.

This challenge tries to explore not just the question of whether LLMs can write code (they can), but the question is more centred around how good the quality of that code is and if they are able to optimise a code for two competing metrics (e.g. speed and accuracy).

Please note: If you are interested in beating my code but writing your own solver, that is fine too, but this must be disclosed in the submission; see details below.

### The challenge

Create a CFD solver that solves the Lid-driven cavity problem at a Reynolds number of 1000. The lid driven cavity is likely one of the first cases every introduced in the context of CFD, and chances are most, if not all of you (except for Sam Altman, of course) will have heard about it and know what it solves.

We have a square domain, with walls on all 4 sides, with the top wall moving at a constant speed from the left to the right. This gives us a vortical flow structure inside the domain, as shown below for the case of a Reynolds number of 1000 [here](https://youtu.be/5zzBzITQRQQ?si=ErI464Fg6PAvt_ru).

I want you to prompt your favourite AI tool to give you a code that solves this exact problem. We use the reference data by [Ghia et al. 1982](https://doi.org/10.1016/0021-9991(82)90058-4) to judge how accurate the simulations are. If you have reservations regarding the choice of this reference (are you from Warwick University, and were you my external PhD examiner?), hold your horses, there is a reason for this, and it will be revealed.

For performance, we will measure your AI code against one that I have written specifically for this challenge, where I have used any technique I can think of to make my code as fast as possible. Can you beat it?

### Constraints

There are constraints in place to ensure we are comparing the same thing. If your solver converges to a different convergence threshold, then comparing accuracy and the time taken is meaningless. Therefore, we have to settle on a few metrics that your solver must respect, given below:

* The solver must be written in C++, but you are free to choose which C++ standard to use. My code is written in C++ and using the same compiler with the same optimisation means we are comparing something which is as close as possible (if that sentence also doesn't sit right with you, I get it, but again, there is a reason for it).
* The domain we use is a square domain in 2D with a constant grid size in the x and y direction, using 129 nodes in each direction (128 cells in each direction). This is to make the mesh size the same compared to the reference we will compare ourselves against.
* The maximum CFL number you can use is 1 (again, specific reason, yada yada yada)
* Your solver has to use CMake for the compilation. This is mainly so we have a unified approach for compiling code, as well as treating optimisations, but also simplifies the compilation of multiple source files if needed. Would now be a good time to selfishly promote my series on [Automating CFD solver and library compilation using CMake](https://cfd.university/learn/automating-cfd-solver-and-library-compilation-using-cmake)?
* You can use as many external dependencies as you wish, but these either need to be handled by CMake if possible (e.g. use [FetchContent](https://cfd.university/learn/automating-cfd-solver-and-library-compilation-using-cmake/how-to-add-external-libraries-into-your-cfd-solver-using-cmake#aioseo-the-modern-way-fetchcontent)) or using Conan as your package manager. If you use Conan, you will need the additional ```conanfile.txt```. Of course, you have a dedicated resource for this as well: [The gold standard: Using a package manager (Conan, not its mentally insane brother, vcpkg!)](https://cfd.university/learn/automating-cfd-solver-and-library-compilation-using-cmake/how-to-add-external-libraries-into-your-cfd-solver-using-cmake#aioseo-the-preferred-way-using-a-package-manager). If a package is not available through [ConanCentre](https://conan.io/center), well, tough luck! I'm not in the mood to compile all sorts of different libraries myself ...
* We are only considering single-core performance. No parallelisation is allowed. This means, no OpenMP, or MPI, or CUDA for GPUs, etc. Extract as much performance out of a single core.

### Required output and code structure

To make sure I can easily compute the performance and accuracy of your solver, there are a few metrics your solver needs to provide. When your solver finishes, it has to generate three files:

- ```uy.csv```
- ```vx.csv```
- ```res.csv```

All of these are CSV files (comma-separated variables). The ```uy.csv``` file must print the u velocity on the vertical centerline, that is, for a $x=0.5$. Using 129 nodes and a uniform grid, this means that if we store ```u``` as a two-dimensional object, we can retrieve this data from ```u[64][j]```. We just need to loop over the y direction and then record the ```y``` value and the ```u``` velocity. An example of the CSV file is shown below:

```
y, u
0, 0
0.1, 0.1
0.2, 0.34
0.3, 0.5
```

Make sure to use the same header ```y, u``` and this order (```y``` first, followed by ```u```). The ```vx.csv``` file, then, is the same for the ```v``` velocity at a constant y value of $y=0.5$. We can extract the v velocity on the horizontal centerline from ```v[i][64]``` by looping over the x direction. Both these CSV files should have 130 lines (129 numerical values + 1 line for the header).

The ```res.csv``` file is the L2-norm of the pressure residual. At each iteration/timestep, you need to compute the residual of the pressure, e.g.

$$
res(p)^{k+1}_{i,j} = p^{k+1}_{i,j} - p^k_{i,j}
$$

where $k$ is the iteration/time level and $i$ and $j$ are the locations in our mesh where the pressure is stored. You then need to write the residual of that pressure to a file, storing the L2-norm of the residual at each iteration/time step. If your solver takes 100 iterations to converge, then your file will have 101 entries (100 numerical values + 1 line for the header). The header should be ```iter, res```.

An example is shown below:

```
iter, res
1, 1
2, 0.5
3, 0.1
4, 0.02
5, 0.0067
```

**Convergence will be judged based on your pressure residual**. We use a modified convergence criterion here, which can be summarised as follows:

$$
\frac{\text{max}(||res(p)||_2)}{||res(p)^{k+1}||_2}
$$

At every iteration/time step, we check if the current computed residual is the largest residual we have recorded thus far. If it is, we update a variable, for example, ```max_res_p``` which stores the highest ever recorded residual. We then compare the residual computed for the current updated solution, i.e. $res(p)^{k+1}$, against the highest residual. If the ratio between the two is $10^{-4}$ or less, then the simulation has converged.

We use the ```res.csv``` file to check that residuals have actually converged to this convergence threshold.

In addition, you need to submit a file called ```README.md```. This file must contain the following structure (please copy and paste directly from here):

```markdown
name:
country:
occupation: 
Have you written a CFD solver by yourself before?:
How long have you been studying/using CFD in years?:
Do you feel you understand the code that was generated?:
Provide a brief description of your solver:
Did you write the solver yourself?:
If not, which LLM did you use?:
```

As an example, this would be the ```README.md``` file for me:

```markdown
name: Tom-Robin Teschner (or, use your Steam gamer name if you prefer to remain anonymous)
country: Germany (why not, let's make it an Olympic sport with medal counting!)
occupation: Senior Lecturer
Have you written a CFD solver by yourself before?: Yes
How long have you been studying/using CFD in years?: 15
Do you feel you understand the code that was generated?: Yes
Provide a brief description of your solver: Well, I'm not telling you, you'll have to wait and see the answer, but you should put info here on, for example, meshing, numerical schemes, pressure-velocity coupling algorithm, etc.
Did you write the solver yourself?: Yes (for most of you, this will be no, as I want as many AI code submissions as possible)
If not, which LLM did you use?: Not applicable
```

Finally, you will need to submit your entire chat history (all of your prompts) that were used to generate the code. You should remove any sensitive information, as this will be placed in a public repository. Name this file ```chat_history.json```.
The chat history should follow standardised JSON formatting for storing chat histories, though a simplified form, as shown below, is acceptable:

```JSON
{
  "chat_history": [
    {
      "id": 1,
      "user_input": "Give me a lid-driven cavity solver at Re=1000 or else ...",
      "response": "Got it, boss, created lid.cpp."
    },
    {
      "id": 2,
      "user_input": "Your stupid code is broken, I have given you VERY CLEAR instructions",
      "response": "I'm on a break right now, come back in 5 ..."
    }
  ]
}
```

Note, typically you would also store user and system roles as well as timestamps, which aren't really relevant here, so using this simplified chat history is fine.

Therefore, your code, once executed, should have the following files in your directory:

```text
your_solver_folder
└── build/ (all CMake garbage goes here)
└── CMakeLists.txt
└── your_solver.cpp
└── conanfile.txt (optional)
└── README.md
└── chat_history.json (ignore this file if you create your own solver)
└── uy.csv  (generated by your_solver.cpp)
└── vx.csv  (generated by your_solver.cpp)
└── res.csv (generated by your_solver.cpp)
```

If any of the required files are missing (e.g. ```chat_history.json```, ```README.md```) or it does not contain the information requested, your submission may not be considered.

### Submission

There are two ways to submit your code. The preferred way is to submit a pull request to this git repository.

If you feel git-challenged, don't worry, zip your folder (clean it up please before zipping, e.g. remove the ```build/``` folder and ensure the folder is complete) and send it by email to tom@cfd.univeristy.

The repository contains a file called ```evaluate.py```. I will use that to automatically run each solver and to report the speed and accuracy. You can test that your solver can be compiled, executed, and analysed by running this file. It requires ```pandas```, ```numpy```, and ```matplotlib``` to work correctly. If you want to use it, make sure your code is placed inside a folder (```evaluate.py``` will look for all folders in the current directory and then go into each folder and compile/execute the code).

The ```evaluate.py``` file will compile your code with default optimisations turned on in CMake, e.g. ```cmake -DCMAKE_BUILD_TYPE=Release``` and all solvers will be subject to the same compiler optimisation here. The performance will measure **total execution time**, meaning that file input and output (e.g. writing the various CSV files) will be part of the timing.

The easiest way to run the code is to use a virtual environment:

* UNIX (Linux, macOS)

```bash
python3 -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
python evaluate.py
```

* Windows

```bash
py -m venv .venv
.\.venv\Scripts\Activate.ps1
pip install -r requirements.txt
py evaluate.py
```

If a ```stats.csv``` file gets created, good, it is working. You should also see the results printed to your screen.

If your code does not work and cannot be compiled/executed by this script, it will be recorded as AI not being able to produce executable code, unless there is a quick and obvious mistake, especially one that suggests that the issue is on my end and not on yours.

### Dissemination

After the submission window closes, I will look through the codes and analyse them for the design decisions the AI was taking. I will also benchmark all solvers against each other and see how well (or poorly) my solver compares to all of the AI submissions.

A leaderboard will be published, showing the best codes in terms of speed and accuracy. Speed will be ranked by execution time, and errors will be ranked from smallest to largest error. Each entry will get a rank for each of these three columns, and the person with the lowest combined rank will lead the leaderboard.

So, if your code has the third shortest execution time, you will be ranked third best here. If you show the lowest error for both u and v velocity, then you will be ranked first in both categories. Your combined rank is 3 + 1 + 1 = 5. For someone who is ranked second in all categories, they would have a total rank of 2 + 2 + 2 = 6. Since 5 is lower than 6, you would be ranked higher overall.

### Timeline

The competition/challenge opens on the **9th of June 2026** and will run for 4 weeks until the **7th of July 2026**. Afterwards, results will be evaluated and disseminated on both GitHub and [cfd.university](https://cfd.university)