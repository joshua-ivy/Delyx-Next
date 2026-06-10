@echo off
setlocal
REM One command to build + run the Delyx desktop app with the embedded CUDA
REM (GPU) model runtime. It loads MSVC's environment so nvcc can find cl.exe,
REM makes sure Node deps are present, then starts the Tauri dev build.

REM --- Locate Visual Studio's MSVC toolset via vswhere (survives VS updates) ---
set "VSWHERE=%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe"
if not exist "%VSWHERE%" (
  echo [dev-cuda] vswhere.exe not found. Install Visual Studio Build Tools with the "Desktop development with C++" workload.
  exit /b 1
)
set "VSINSTALL="
for /f "usebackq tokens=*" %%i in (`"%VSWHERE%" -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath`) do set "VSINSTALL=%%i"
if not defined VSINSTALL (
  echo [dev-cuda] No Visual Studio C++ x64 toolset found. Install the "Desktop development with C++" workload.
  exit /b 1
)
set "VCVARS=%VSINSTALL%\VC\Auxiliary\Build\vcvars64.bat"
if not exist "%VCVARS%" (
  echo [dev-cuda] vcvars64.bat not found at "%VCVARS%".
  exit /b 1
)
echo [dev-cuda] Loading MSVC environment from "%VSINSTALL%"...
call "%VCVARS%" >nul 2>nul || (echo [dev-cuda] Failed to initialize the MSVC environment.& exit /b 1)

REM --- Verify the CUDA compiler is reachable (mistralrs kernels need nvcc) ---
where nvcc >nul 2>&1 || (echo [dev-cuda] nvcc not found on PATH. Install the NVIDIA CUDA Toolkit.& exit /b 1)

REM --- Ensure Node dependencies are installed ---
if not exist "node_modules" (
  echo [dev-cuda] Installing Node dependencies...
  call ".\.tools\npm.cmd" install || (echo [dev-cuda] npm install failed.& exit /b 1)
)

REM --- Build + run the desktop app (embedded_mistral_cuda) ---
echo [dev-cuda] Building and launching the desktop app (GPU runtime)...
call ".\.tools\npm.cmd" run dev:desktop:cuda
exit /b %errorlevel%
