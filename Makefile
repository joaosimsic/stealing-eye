OUTPUT ?=
FPS ?=
MIN_CLIP ?=
MIN_DURATION ?=
MIN_AREA ?=
STOP_DEBOUNCE ?=
AUDIO ?=

.PHONY: build run login check

build:
	cargo build --release

check:
	@[ -n "$(OUTPUT)" ] && [ -n "$(FPS)" ] && [ -n "$(MIN_CLIP)" ] && [ -n "$(MIN_DURATION)" ] && [ -n "$(MIN_AREA)" ] && [ -n "$(STOP_DEBOUNCE)" ] && [ -n "$(AUDIO)" ] || { echo "make run: set OUTPUT FPS MIN_CLIP MIN_DURATION MIN_AREA STOP_DEBOUNCE AUDIO"; exit 1; }

run: check build
	cd automation && npm run record -- --output "$(OUTPUT)" --fps "$(FPS)" --min-clip "$(MIN_CLIP)" --min-duration "$(MIN_DURATION)" --min-area "$(MIN_AREA)" --stop-debounce "$(STOP_DEBOUNCE)" --$(AUDIO)

login:
	cd automation && npm run login
