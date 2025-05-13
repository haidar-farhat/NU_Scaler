from setuptools import setup, find_packages

setup(
    name="nu_scaler",
    version="0.1.0",
    packages=find_packages(),
    install_requires=[
        "PySide6",
        "Pillow",
        "numpy",
        "ffpyplayer",
        "imageio",
        "psutil",
        "pywin32",
    ],
    python_requires=">=3.8",
) 