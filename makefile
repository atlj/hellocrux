MKV_FILES := $(wildcard *.mkv)
MOV_FILES := $(patsubst %.mkv,%.mov,$(MKV_FILES))

.PHONY: format
format:
	cargo clippy --fix --allow-dirty && \
	cargo fmt --all & \
	swiftlint lint --fix ios/**/*.swift && \
	swiftformat ./**/*.swift --swiftversion 6.2

.PHONY: convert_mkv
convert_mkv: $(MOV_FILES)

.PHONY: clean_mov
clean_mov:
	@echo "Cleaning up .mov files..."
	rm -f *.mov

%.mov: %.mkv
	@echo "Converting $< to $@"
	ffmpeg -i $< -c copy $@
