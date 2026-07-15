import os
import sys
import shutil
import subprocess

def run_cmd(args, env=None):
    print(f"Running: {' '.join(args)}")
    result = subprocess.run(args, capture_output=True, text=True, env=env)
    if result.returncode != 0:
        print("STDOUT:")
        print(result.stdout)
        print("STDERR:")
        print(result.stderr)
        sys.exit(result.returncode)
    return result.stdout

def main():
    # Resolve workspace directory dynamically relative to the script location
    workspace = os.path.dirname(os.path.abspath(__file__))
    venv_dir = os.path.join(workspace, "black_box_venv")
    
    # 1. Clean previous venv if exists
    if os.path.exists(venv_dir):
        print(f"Cleaning existing venv: {venv_dir}")
        shutil.rmtree(venv_dir)
        
    # 2. Create clean virtual environment using uv
    run_cmd(["uv", "venv", venv_dir])
    
    # 3. Determine python executable in venv
    venv_python = os.path.join(venv_dir, "bin", "python")
    
    # 4. Install our local package along with test runner 'virtue' and dependencies
    print("Installing jsonschema-fast, virtue, and jsonpath-ng into the virtual environment...")
    run_cmd(["uv", "pip", "install", "--python", venv_python, f"{workspace}[format]", "virtue", "jsonpath-ng"])
    
    # 5. Check version installed
    version_out = run_cmd([venv_python, "-c", "import jsonschema_fast; print(jsonschema_fast.__version__)"])
    print(f"Installed package version: {version_out.strip()}")
    
    # 6. Run the tests in the black box environment
    print("\nRunning the tests on the installed package...")
    test_env = os.environ.copy()
    test_env["JSON_SCHEMA_TEST_SUITE"] = "json"
    
    virtue_bin = os.path.join(venv_dir, "bin", "virtue")
    
    result = subprocess.run([virtue_bin, "jsonschema_fast"], env=test_env)
    
    # Clean up venv
    if os.path.exists(venv_dir):
        print(f"Cleaning up virtual environment: {venv_dir}")
        shutil.rmtree(venv_dir)

    if result.returncode == 0:
        print("\nSUCCESS: All tests passed on the installed jsonschema_fast package!")
    else:
        print(f"\nFAILURE: Test suite failed with exit code {result.returncode}")
        sys.exit(result.returncode)

if __name__ == "__main__":
    main()
