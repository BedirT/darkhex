"""Smoke test: verify the Rust engine module loads correctly."""


def test_engine_imports():
    from darkhex._engine import __version__

    assert __version__ == "0.1.0"
