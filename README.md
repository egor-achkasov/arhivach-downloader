# Русская версия внизу / Russian version below

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

---

# Загрузчик Архивача

Скачивает треды с arhivach.vc и сохраняет их локально для офлайн-доступа или архивирования.

# Использование

## TUI

Запустите arhivach-downloader-tui, вставьте ссылку в первое поле, заполните аргументы (описание см. в подразделе CLI) и нажмите Enter.

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

Создаёт DIR и записывает следующее содержимое:
- `index.html` — тред. Откройте его в браузере.
- `files/` (если указан флаг `-f`/`--files`) — оригинальные файлы из постов. Может быть большим, если в треде много видео.
- `thumb/` (если указан флаг `-t`/`--thumb`) — миниатюры, необходимые для отображения превью файлов в треде.

Используйте `-r`/`--resume`, чтобы пропускать уже скачанные файлы и миниатюры. Пропускаются даже частично скачанные или повреждённые файлы.

Используйте `-d`/`--dir`, чтобы указать, куда создать папку треда (по умолчанию — текущая директория).

Используйте `-R`/`--retries`, чтобы задать количество повторных попыток при ошибке скачивания (по умолчанию: 3).
