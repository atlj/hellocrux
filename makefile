MKV_FILES := $(wildcard *.mkv)
MOV_FILES := $(patsubst %.mkv,%.mov,$(MKV_FILES))

.PHONY: all clean

all: $(MOV_FILES)

%.mov: %.mkv
	@echo "Converting $< to $@"
	ffmpeg -i $< -c copy $@

clean:
	@echo "Cleaning up .mov files..."
	rm -f *.mov
