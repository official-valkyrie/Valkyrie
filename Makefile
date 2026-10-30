BIN = valkyrie
EXT = $(if $(filter Windows_NT,$(OS)),.exe,)

HASH         = \#
VERSION      = $(lastword $(subst @, ,$(lastword $(subst $(HASH), ,$(shell cargo pkgid -p valkyrie)))))
RELEASE_NAME = Valkyrie-v$(VERSION)
RELEASE_DIR  = releases/$(RELEASE_NAME)

comma := ,
space := $(subst x,,x x)

CARGO_FLAGS =
TARGET_FEATURES_BASE =
TARGET_FEATURES =
CPU_FLAGS   = -C target-cpu=$(TARGET_CPU)$(if $(strip $(TARGET_FEATURES_BASE) $(TARGET_FEATURES)), -C target-feature=$(subst $(space),$(comma),$(strip $(TARGET_FEATURES_BASE) $(TARGET_FEATURES))))
FEATURES    =
DEBUG       = $(filter debug,$(MAKECMDGOALS))
DEBUG_FLAGS = $(if $(DEBUG),--features debug --config profile.release.debug=true --config profile.release.strip=false)
EXE         = $(BIN)
SRC         = target/release/$(BIN)$(EXT)
DEST        = $(EXE)$(EXT)
DEST_DIR    = .

ifeq ($(OS),Windows_NT)
    SET_RUSTFLAGS = set RUSTFLAGS=$(CPU_FLAGS) &&
    MKDIR_CMD     = if not exist "$(subst /,\,$(DEST_DIR))" mkdir "$(subst /,\,$(DEST_DIR))"
    COPY_CMD      = copy /Y "$(subst /,\,$(SRC))" "$(subst /,\,$(DEST))" >nul
else
    SET_RUSTFLAGS = RUSTFLAGS="$(CPU_FLAGS)"
    MKDIR_CMD     = mkdir -p "$(DEST_DIR)"
    COPY_CMD      = cp "$(SRC)" "$(DEST)"
endif

RELEASE_TARGETS = v2 v3-nopext v3 v4

.PHONY: default debug v2 v3 v4 native-nopext v3-nopext release $(addprefix release-,$(RELEASE_TARGETS))
.NOTPARALLEL:

debug: $(if $(filter-out debug,$(MAKECMDGOALS)),,default) ;

default: TARGET_CPU = native
v2:      TARGET_CPU = x86-64-v2
v3:      TARGET_CPU = x86-64-v3
v4:      TARGET_CPU = x86-64-v4

native-nopext: TARGET_CPU = native
native-nopext: TARGET_FEATURES_BASE = -bmi2
v3-nopext:     TARGET_CPU = x86-64-v3
v3-nopext:     TARGET_FEATURES_BASE = -bmi2

default v2 v3 v4 native-nopext v3-nopext:
	$(SET_RUSTFLAGS) cargo build --release $(CARGO_FLAGS) $(DEBUG_FLAGS) $(if $(FEATURES),--features $(FEATURES)) -p valkyrie --bin $(BIN)
	$(COPY_CMD)

release-v2:        TARGET_CPU = x86-64-v2
release-v3-nopext: TARGET_CPU = x86-64-v3
release-v3-nopext: TARGET_FEATURES_BASE = -bmi2
release-v3:        TARGET_CPU = x86-64-v3
release-v4:        TARGET_CPU = x86-64-v4
release-%:         CARGO_FLAGS = --features release
release-%:         override TARGET_FEATURES =
release-%:         override FEATURES =
release-%:         DEST_DIR = $(RELEASE_DIR)
release-%:         DEST = $(RELEASE_DIR)/$(RELEASE_NAME)-$(patsubst release-%,%,$@)$(EXT)

release: $(addprefix release-,$(RELEASE_TARGETS))

$(addprefix release-,$(RELEASE_TARGETS)):
	$(MKDIR_CMD)
	$(SET_RUSTFLAGS) cargo build --release $(CARGO_FLAGS) -p valkyrie --bin $(BIN)
	$(COPY_CMD)
