# Matlab FMU

## Installation Instructions

To use this FMU, you need:
- an installation of matlab
- a python interpreter with the following
  - matlab engine installed: [instructions](https://se.mathworks.com/help/matlab/matlab_external/install-the-matlab-engine-for-python.html)
  - packages described in [requirements.txt](./requirements.txt) installed
  
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
