from pathlib import Path
import sys
import re
from os import getcwd, path, system, chdir, remove
from shutil import copyfile
from colorama import Fore

test_files = []
start_dir = getcwd()
test_dir = f"{start_dir}/test/env"

# Get all Rust files in the project which contain a test module.
for file in Path("lab_os").rglob("*.rs"):
    with open(file, "r") as f:
        for line in f:
            if re.search(r"^#\[cfg\(test\)\]", line):
                test_files.append(file)
        f.close()

if len(test_files) == 0:
    print("Nothing to do.")
    sys.exit(0)

results = []

for file in test_files:
    print("\n" + ("=" * 20) + "\n")
    print(f"Running tests in {str(file)}...\n")

    # Get the files basename and the module within it.
    # e.g. './src/driver.rs' -> 'driver.rs' and 'driver'
    name = str(path.basename(file))
    module = name.replace(".rs", "")

    # Create a boilerplate main.rs to wrap the test module.
    main_rs = f"{test_dir}/main.rs"
    with open(main_rs, "w") as f:
        f.write(f"mod {module}; fn main() {{}}")
        f.close()

    # Copy the file from the project into the testing environment.
    temp_file = f"{test_dir}/{name}"
    copyfile(file, temp_file)

    # Run the tests and record the results.
    chdir(test_dir)
    success = system("cargo test") == 0
    results.append((name, success))

    # Clean up.
    chdir(start_dir)
    remove(temp_file)
    remove(main_rs)

    print("\n" + ("=" * 20) + "\n")

num_success = len([r for r in results if r[1]])
num_fail = len([r for r in results if not r[1]])

print(f"Summary: {num_success} modules passed; {num_fail} modules failed", end="\n\n\t")
for res in results:
    print(f"{res[0]} ... ", end="")
    if res[1]:
        print(Fore.GREEN + "ok" + Fore.RESET, end="\n\t")
    else:
        print(Fore.RED + "failed" + Fore.RESET, end="\n\t")
print("")
