SRC := $(wildcard *.c)

OBJ := ${SRC:.c=.o}

TARGETDIR := ../
TARGET := $(TARGETDIR)/libespresso.a

all: prepare $(TARGET)

prepare:
	mkdir -p $(TARGETDIR)

$(TARGET): $(OBJ)
	ar -rc $(TARGET) $(OBJ)

clean: 
	@- $(RM) $(OBJ)

