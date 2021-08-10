# Matlab FMU

## Installation Instructions

To use this FMU, you need:
- an installation of matlab
- a python interpreter with the following
  - matlab engine installed (instructions taken from [here](https://se.mathworks.com/help/matlab/matlab_external/install-the-matlab-engine-for-python.html))
    1. Make sure you have a python that's compatible with your matlab.
    2. Open an administrator's terminal at the Matlab's python installation directory. For example: C:\Program Files\MATLAB\R2021a\extern\engines\python
    3. Install the matlab engine (use `sudo` if on linux): `python .\setup.py install`
    4. Test the matlab engine by starting a python interpreter and running
        ```
        >>> import matlab.engine
        >>> eng = matlab.engine.start_matlab()
        >>> eng.plus(3,4)
        7
        ```
  - packages described in [requirements.txt](./requirements.txt) installed
    1. Open a terminal in the current folder, and run: `pip install -r requirements.txt`

## Editing and Launching the Matlab Code

The matlab code needs to be placed inside the [matlabcode](./matlabcode) folder.
An example "Adder FMU" is provided.

The [launch.toml](./launch.toml) folder controls how to start the [matlab_proxy.py](./matlab_proxy.py) script, 
which is the one that will automatically start and interact with matlab, when the FMU is instantiated.

Workflow to update the FMU:
1. Change the matlab code inside [matlabcode](./matlabcode) folder
2. Update the [modelDescription.xml](./modelDescription.xml) according to the matlab code.
  1. Pay attention to the value references of the variables, and make sure these are consistent with the matlab code.
3. After the changes, zip the folder and change the extension to FMU.
