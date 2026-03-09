# Arhivach downloader

Download threads from arhivach.vc and save them locally for offline access or preservation.

# Usage

## TUI

Run arhivach-downloader-tui, paste the url in the first field, fill the arguments (see description the the CLI subsection) and press Enter.

## CLI

`arhivach-downloader-cli --help`:

```
Download threads from arhivach.

Usage: arhivach-downloader-cli [OPTIONS] <URL>

Arguments:
  <URL>  URL to download

Options:
  -d, --dir <DIR>                   Path to download directory [default: .]
  -e, --exporter <EXPORTER>         Exporter [default: html] [possible values: html]
  -t, --thumb                       Download thumbnail images, default: false
  -f, --files                       Download files (images, videos, gifs, etc), default: false
  -r, --resume                      Resume files and thumbnails downloading instead of overwriting. Useless if neither -t nor -f are set, default: false
  -R, --retries <DOWNLOAD_RETRIES>  Download retries in case of a error [default: 3]
  -h, --help                        Print help
```

Creates DIR and writes the following content:
- `index.html` — the thread. Open it with your web browser.
- `files/` (if `-f`/`--files` is given) — original files attached to posts. May be large if there are many videos.
- `thumb/` (if `-t`/`--thumb` is given) — thumbnails needed to render file previews in the thread.

Use `-r`/`--resume` to skip files and thumbnails that are already downloaded. Skips them even if they are partially downloaded or corrupted.

Use `-d`/`--dir` to specify where to create the thread directory (defaults to the current directory).

Use `-R`/`--retries` to control how many times a failed download is retried (default: 3).
