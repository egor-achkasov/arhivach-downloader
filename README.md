# Arhivach downloader

Download threads from arhivach.vc and save them locally for offline access or preservation.

# Usage

## CLI

`arhivach-downloader --help`:

```
Download threads from arhivach.

Usage: arhivarch-downloader.exe [OPTIONS] [URL]

Arguments:
  [URL]  URL to download

Options:
  -l, --list <LIST>  Path to a text file containing a list of URLs (one per line)
  -t, --thumb        Download thumbnail images, default: false
  -f, --files        Download files (images, videos, gifs, etc), default: false
  -r, --resume       Resume files and thumbnails downloading instead of overwriting. Useless if neither -t nor -f are set, default: false
  -h, --help         Print help
```

Each thread will be downloaded in a directory named by an OP №. Contents:
- index.html -- the thread. Open it with your web browser.
- files directory (if -f (--files) argument is given) -- all the files original attached to posts. Might be heavy if there are many videos.
- thumb directory (if -t (--thumb) argument is given) -- all the thumbnails needed to render file previews in the thread.

Main index.html will be created in the current directory to feature the first posts of the downloaded threads.

Note that you may pass an URL directly as an argument, pass a path to a text file with URLs via -f, or both.

Use -r (--resume) to skip downloading files and thumbnails that are already there.
