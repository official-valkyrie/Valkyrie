EXE = valkyrie
EXT = $(if $(filter Windows_NT,$(OS)),.exe,)

ifeq ($(OS),Windows_NT)
    SET_RUSTFLAGS = set RUSTFLAGS=-C target-cpu=$(TARGET_CPU) $(EXTRA_FLAGS) &&
    COPY_CMD      = copy /Y "target\release\$(EXE)$(EXT)" "$(EXE)$(EXT)" >nul 2>&1
else
    SET_RUSTFLAGS = RUSTFLAGS="-C target-cpu=$(TARGET_CPU) $(EXTRA_FLAGS)"
    COPY_CMD      = cp "target/release/$(EXE)$(EXT)" "./$(EXE)$(EXT)" 2>/dev/null
endif

.PHONY: default debug v2 v3 v4 native-nopext v3-nopext

default: TARGET_CPU = native
debug:   TARGET_CPU = native
v2:      TARGET_CPU = x86-64-v2
v3:      TARGET_CPU = x86-64-v3
v4:      TARGET_CPU = x86-64-v4

native-nopext: TARGET_CPU = native
native-nopext: EXTRA_FLAGS = -C target-feature=-bmi2
v3-nopext:     TARGET_CPU = x86-64-v3
v3-nopext:     EXTRA_FLAGS = -C target-feature=-bmi2

default v2 v3 v4 native-nopext v3-nopext:
	$(SET_RUSTFLAGS) cargo build --release -p valkyrie --bin valkyrie
	$(COPY_CMD)

debug:
	$(SET_RUSTFLAGS) cargo build -p valkyrie --bin valkyrie --features=debug
	$(COPY_CMD)