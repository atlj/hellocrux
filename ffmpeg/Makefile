.PHONY: all tests/fixtures/h265_with_data.mp4 tests/fixtures/h265_with_attachment.mkv
all: tests/fixtures/h265_with_data.mp4 tests/fixtures/h265_with_attachment.mkv

tests/fixtures/h265_with_data.mp4:
	ffmpeg \
		-f "lavfi" -i "testsrc" \
		-c:v libx265 \
		-timecode 00:00:00:00 \
		-t 100ms \
		-y \
		tests/fixtures/h265_with_data.mp4

tests/fixtures/h265_with_attachment.mkv:
	ffmpeg \
		-f "lavfi" -i "testsrc" \
		-c:v libx265 \
		-attach "./tests/fixtures/attachment.json" -metadata:s:t mimetype=application/json \
		-t 100ms \
		-y \
		tests/fixtures/h265_with_attachment.mkv

