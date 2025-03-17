from setuptools import find_packages, setup

setup(
    name='controller_framework',
    version='0.0.1',
    description='A framework to simplify creation of controllers',
    author='Higor de Lima',
    packages=find_packages(include=['controller_framework', 'controller_framework.*']),
    python_requires='>=3.7',
    classifiers=[
        'Development Status :: 3 - Alpha',
        'Intended Audience :: Developers',
        'Programming Language :: Python :: 3',
        'Programming Language :: Python :: 3.7',
        'Programming Language :: Python :: 3.8',
        'Programming Language :: Python :: 3.9',
        'Programming Language :: Python :: 3.10',
        'Operating System :: OS Independent',
    ],
)
