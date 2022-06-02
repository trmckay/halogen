PDF := Report.pdf

all: $(PDF)

clean:
	rm -f $(PDF)

%.pdf: %.md
	pandoc $< -o $@

.PHONY: all clean

