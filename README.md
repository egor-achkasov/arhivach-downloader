# Arhivach downloader

Download threads from arhivach.vc and save them locally for offline access or preservation.

# Usage

## CLI

`arhivach-downloader --help`:

```
Usage: arhivarch-downloader-cli.exe [OPTIONS] <URL>

Arguments:
  <URL>  URL to download

Options:
  -d, --dir <DIR>            Path to download directory [default: .]
  -e, --exporter <EXPORTER>  Exporter [default: html] [possible values: html]
  -t, --thumb                Download thumbnail images, default: false
  -f, --files                Download files (images, videos, gifs, etc), default: false
  -r, --resume               Resume files and thumbnails downloading instead of overwriting. Useless if neither -t nor -f are set, default: false
  -R, --retries <RETRIES>    Download retries in case of a error [default: 3]
  -h, --help                 Print help
```

Creates a subdirectory named after the arhivach thread id (the number after `/thread/` in the URL) inside the download directory, and saves the thread there. Contents:
- `index.html` — the thread. Open it with your web browser.
- `files/` (if `-f`/`--files` is given) — original files attached to posts. May be large if there are many videos.
- `thumb/` (if `-t`/`--thumb` is given) — thumbnails needed to render file previews in the thread.

Use `-r`/`--resume` to skip files and thumbnails that are already downloaded.

Use `-d`/`--dir` to specify where to create the thread directory (defaults to the current directory).

Use `-R`/`--retries` to control how many times a failed download is retried (default: 3).
