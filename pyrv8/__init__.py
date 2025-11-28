"""
PyRV8
-----
Python rust bindings for Rustscript and deno-js allowing
for easy javascript cracking and captcha solving.
"""
from .pyrv8 import *

if hasattr(pyrv8, "__all__"):
    __all__ = pyrv8.__all__
