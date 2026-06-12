from os import listdir, makedirs
from os.path import isdir, join
import subprocess
from pathlib import Path
import time
import platform

import matplotlib.pyplot as plt
import pandas as pd
import numpy as np

def main():
    # get all folders in current folder but exclude any files from the list
    folders = [f for f in listdir('.') if isdir(f)]

    stats = pd.DataFrame(columns=['folder', 'execution_time [s]', 'u_rsme [%]', 'v_rsme [%]', 'residual'])
    execution_times = []
    u_rsme = []
    v_rsme = []
    residual = []
    all_folders = []

    # perform checks
    for index, folder in enumerate(folders):
        print(f'\033[33mChecking folder: {folder} ({index+1}/{len(folders)})\033[0m')

        # check that single source file is present
        check_for_cmake_project(folder)

        # compile code
        compile_code(folder)

        # execute code
        execution_time = execute_code(folder)

        # plot results
        ursme, vrsme = analyse_results(folder)
        res = analyse_residuals(folder)

        # add statistics
        all_folders.append(folder)
        execution_times.append(execution_time)
        u_rsme.append(ursme * 100)
        v_rsme.append(vrsme * 100)
        residual.append(res)

    stats['folder'] = all_folders
    stats['execution_time [s]'] = execution_times
    stats['u_rsme [%]'] = u_rsme
    stats['v_rsme [%]'] = v_rsme
    stats['residual'] = residual

    print(stats)

    stats.to_csv('stats.csv', index=False)


def check_for_cmake_project(folder):
    files = [f for f in listdir(folder)]
    check_passed = 'CMakeLists.txt' in files
    print_task_output(1, 'Check for CMake project', check_passed)

def compile_code(folder):
    # create build folder for CMake
    makedirs(f'{folder}/build', exist_ok=True)

    # configure and compile instructions for CMake
    configure_cmd = ['cmake', '-B', 'build', '-DCMAKE_BUILD_TYPE=Release', '-G', 'Ninja']
    compile_cmd = ['cmake', '--build', 'build', '--config', 'Release']

    configure_out = subprocess.run(configure_cmd, cwd=folder, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    compile_out = subprocess.run(compile_cmd, cwd=folder, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    check_passed = configure_out.returncode == 0 and compile_out.returncode == 0
    print_task_output(2, 'Compile code', check_passed)

def execute_code(folder):
    # read executable name from CMakeLists.txt
    with open(f'{folder}/CMakeLists.txt', 'r') as f:
        lines = f.readlines()

    executable_name = ''
    for line in lines:
        if 'add_executable(' in line:
            executable_name = line.replace('add_executable(', '')
            executable_name = executable_name.split(' ')[0]
            break
    assert executable_name != '', 'Executable name not found in CMakeLists.txt'

    # check operating system
    if platform.system().lower() == 'windows':
        executable_name += '.exe'

    executable_name = join(folder, 'build', executable_name)

    # execute the code
    start = time.perf_counter()
    out = subprocess.run([str(executable_name)], cwd=folder, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    end = time.perf_counter()
    check_passed = out.returncode == 0
    print_task_output(3, 'Execute code', check_passed)

    # write log
    with open(f'{folder}/stdout.txt', 'w') as f:
        f.write(out.stdout.decode('utf-8'))
    with open(f'{folder}/stderr.txt', 'w') as f:
        f.write(out.stderr.decode('utf-8'))

    return end - start

def print_task_output(task_id, check_text, passed):
    check_length = len(check_text)
    num_dots = 36 - check_length

    print(f'\033[34mTask {task_id}:\033[0m {check_text} {"." * num_dots}', end='')

    if passed:
        print('\033[32m OK \033[0m')
    else:
        print('\033[31m ERROR \033[0m')

def analyse_results(folder):
    # ghia et al. data
    ref = {
        'uy': {
            'u': [ 1.00000, 0.65928, 0.57492, 0.51117, 0.46604, 0.33304, 0.18719, 0.05702,-0.06080,-0.10648,-0.27805,-0.38289,-0.29730,-0.22220,-0.20196,-0.18109, 0.00000],
            'y': [1.0000, 0.9766, 0.9688, 0.9609, 0.9531, 0.8516, 0.7344, 0.6172, 0.5000, 0.4531, 0.2813, 0.1719, 0.1016, 0.0703, 0.0625, 0.0547, 0.0000]
        },
        'vx': {
            'v': [ 0.00000, -0.21388, -0.27669, -0.33714, -0.39188, -0.51550, -0.42665, -0.31966,  0.02526,  0.32235,  0.33075,  0.37095,  0.32627,  0.30353,  0.29012,  0.27485,  0.00000],
            'x': [1.0000, 0.9688, 0.9609, 0.9531, 0.9453, 0.9063, 0.8594, 0.8047, 0.5000, 0.2344, 0.2266, 0.1563, 0.0938, 0.0781, 0.0703, 0.0625, 0.0000]
        }
    }

    # simulation data
    uy = pd.read_csv(f'{folder}/uy.csv')
    vx = pd.read_csv(f'{folder}/vx.csv')

    # strip whiespaces from headers
    uy.columns = [x.strip() for x in uy.columns]
    vx.columns = [x.strip() for x in vx.columns]

    # normalise data
    ref['vx']['x'] = [2 * x - 1 for x in ref['vx']['x']]
    ref['uy']['y'] = [2 * y - 1 for y in ref['uy']['y']]

    vx['x'] = [2 * x - 1 for x in vx['x']]
    uy['y'] = [2 * y - 1 for y in uy['y']]

    # plot data
    plt.plot(vx['x'], vx['v'], '-b', label='Simulation')
    plt.plot(uy['u'], uy['y'], '-b')
    plt.plot(ref['vx']['x'], ref['vx']['v'], 'o', markerfacecolor='none', markeredgecolor='black', markersize=7, label='Ghia et al.')
    plt.plot(ref['uy']['u'], ref['uy']['y'], 'o', markerfacecolor='none', markeredgecolor='black', markersize=7)
    plt.legend(loc='best')
    plt.savefig(f'{folder}/lid.png')
    plt.close()

    # evaluate error as well
    v_int = np.interp(ref['vx']['x'], vx['x'], vx['v'])
    u_int = np.interp(ref['uy']['y'], uy['y'], uy['u'])

    v_error = ref['vx']['v'] - v_int
    u_error = ref['uy']['u'] - u_int

    v_rmse = np.sqrt(np.mean(v_error**2))
    u_rmse = np.sqrt(np.mean(u_error**2))

    return u_rmse, v_rmse


def analyse_residuals(folder):
    # simulation data
    res = pd.read_csv(f'{folder}/res.csv')

    # strip whiespaces from headers
    res.columns = [x.strip() for x in res.columns]

    # plot data
    plt.semilogy(res['iter'], res['res'], '-b', label='pressure')
    plt.legend(loc='best')
    plt.savefig(f'{folder}/residuals.png')

    # remove any rows where the residual is zero for data cleaning
    res = res[res['res'] != 0]

    # find the highest and lowest residual
    highest = res['res'].max()
    lowest = res['res'].min()

    return lowest / highest


if __name__ == '__main__':
    main()