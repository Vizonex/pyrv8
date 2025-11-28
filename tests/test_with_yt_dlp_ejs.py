# Thought this would be an intresting one to test this on doing so is completely optional.
import pytest
from pyrv8 import Context

yt_dlp_ejs = pytest.importorskip("yt_dlp_ejs.yt.solver")

def test_ejs_solver_can_be_loaded() -> Context:
    l = yt_dlp_ejs.lib()
    c = yt_dlp_ejs.core()
    ctx = Context()
    ctx.eval(f"""
    {l}
    // Silly little rearrangement does the trick
    var meriyah = lib.meriyah;
    var astring = lib.astring;
    {c}
    """)


