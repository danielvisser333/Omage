import shader
import pathlib
import shutil;
import os;

if __name__ == '__main__':
    current_dir = pathlib.Path(__file__).parent.absolute()
    if not os.path.isdir(current_dir.joinpath("./target")):
        os.mkdir(current_dir.joinpath("./target"))
    if not os.path.isdir(current_dir.joinpath("./target/release/")):
        os.mkdir(current_dir.joinpath("./target/release/"))
    if not os.path.isdir(current_dir.joinpath("./target/debug/")):
        os.mkdir(current_dir.joinpath("./target/debug/"))
    if not os.path.isdir(current_dir.joinpath("./target/release/assets")):
        os.mkdir(current_dir.joinpath("./target/release/assets"))
    if not os.path.isdir(current_dir.joinpath("./target/debug/assets")):
        os.mkdir(current_dir.joinpath("./target/debug/assets"))
    shader.compile_shaders()
    current_dir = pathlib.Path(__file__).parent.absolute()
    assets_dir = current_dir.joinpath("./assets")
    for root, _, files in os.walk(assets_dir):
        for file in files:
            shutil.copy(current_dir.joinpath("./assets/"+file), current_dir.joinpath("./target/debug/assets/" + file))
            shutil.copy(current_dir.joinpath("./assets/"+file), current_dir.joinpath("./target/release/assets/" + file))
