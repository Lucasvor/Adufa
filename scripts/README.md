# Asset generation

`generate_brand_assets.py` creates the committed raster icons, Windows `.ico`
package, and deterministic README demonstration GIF from one visual definition.
It is kept in the repository so documentation assets can be reviewed and
regenerated without a personal desktop recording.

Requirements:

```powershell
python -m pip install Pillow
```

Generate all assets from the repository root:

```powershell
python scripts/generate_brand_assets.py
```

Review the resulting files in `docs/assets/` and `platforms/windows/assets/`
before committing them. The script never reads audio data or captures the
desktop.