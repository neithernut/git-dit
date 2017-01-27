PANDOC=$(shell which pandoc)

all: man

man:
	@$(PANDOC) -s -t man git-dit.1.md -o git-dit.1

