BIN = valkyrie
EXT = $(if $(filter Windows_NT,$(OS)),.exe,)

HASH            = \#
VERSION         = $(lastword $(subst @, ,$(lastword $(subst $(HASH), ,$(shell cargo pkgid -p valkyrie)))))
COMMIT          = $(or $(shell git log -1 --format=%cd-%h --date=format:%Y%m%d --abbrev=8),nogit)
RELEASE_NAME    = valkyrie-$(VERSION)
PRERELEASE_NAME = valkyrie-prerelease-$(COMMIT)

comma := ,
space := $(subst x,,x x)

HOST        := $(shell rustc --print host-tuple)
CARGO_FLAGS = --target $(HOST)
TARGET_FEATURES_BASE =
TARGET_FEATURES =
CPU_FLAGS   = -C target-cpu=$(TARGET_CPU)$(if $(strip $(TARGET_FEATURES_BASE) $(TARGET_FEATURES)), -C target-feature=$(subst $(space),$(comma),$(strip $(TARGET_FEATURES_BASE) $(TARGET_FEATURES))))
FEATURES    =
DEBUG       = $(filter debug,$(MAKECMDGOALS))
DEBUG_FLAGS = $(if $(DEBUG),--features debug --config profile.release.debug=true --config profile.release.strip=false)
EXE         = $(BIN)
SRC         = target/$(HOST)/release/$(BIN)$(EXT)
DEST        = $(EXE)$(EXT)
DEST_DIR    = .

ifeq ($(OS),Windows_NT)
    SHELL         := cmd.exe
    PLATFORM      = windows
    SET_RUSTFLAGS = set RUSTFLAGS=$(CPU_FLAGS) &&
    MKDIR_CMD     = if not exist "$(subst /,\,$(DEST_DIR))" mkdir "$(subst /,\,$(DEST_DIR))"
    COPY_CMD      = copy /Y "$(subst /,\,$(SRC))" "$(subst /,\,$(DEST))" >nul
else
    PLATFORM      = $(shell uname -s | tr A-Z a-z)
    SET_RUSTFLAGS = RUSTFLAGS="$(CPU_FLAGS)"
    MKDIR_CMD     = mkdir -p "$(DEST_DIR)"
    COPY_CMD      = cp "$(SRC)" "$(DEST)"
endif

PACKAGE_TARGETS = v2 v3-nopext v3 v4
RELEASES        = $(addprefix release-,$(PACKAGE_TARGETS))
PRERELEASES     = $(addprefix prerelease-,$(PACKAGE_TARGETS))
flavors         = $(1) release-$(1) prerelease-$(1)

.PHONY: default debug v2 v3 v4 native-nopext v3-nopext release prerelease $(RELEASES) $(PRERELEASES)
.NOTPARALLEL:

debug: $(if $(filter-out debug,$(MAKECMDGOALS)),,default) ;

default:                   TARGET_CPU = native
native-nopext:             TARGET_CPU = native
native-nopext:             TARGET_FEATURES_BASE = -bmi2
$(call flavors,v2):        TARGET_CPU = x86-64-v2
$(call flavors,v3-nopext): TARGET_CPU = x86-64-v3
$(call flavors,v3-nopext): TARGET_FEATURES_BASE = -bmi2
$(call flavors,v3):        TARGET_CPU = x86-64-v3
$(call flavors,v4):        TARGET_CPU = x86-64-v4

default v2 v3 v4 native-nopext v3-nopext:
	$(SET_RUSTFLAGS) cargo build --release $(CARGO_FLAGS) $(DEBUG_FLAGS) $(if $(FEATURES),--features $(FEATURES)) -p valkyrie --bin $(BIN)
	$(COPY_CMD)

$(RELEASES):                PACKAGE = $(RELEASE_NAME)
$(RELEASES):                PACKAGE_FEATURES = release
$(PRERELEASES):             PACKAGE = $(PRERELEASE_NAME)
$(PRERELEASES):             PACKAGE_FEATURES = prerelease
$(RELEASES) $(PRERELEASES): override TARGET_FEATURES =
$(RELEASES) $(PRERELEASES): DEST_DIR = releases/$(PACKAGE)
$(RELEASES) $(PRERELEASES): DEST = $(DEST_DIR)/$(PACKAGE)-$(PLATFORM)-x86-64-$(patsubst prerelease-%,%,$(patsubst release-%,%,$@))$(EXT)

release:    $(RELEASES)
prerelease: $(PRERELEASES)

$(RELEASES) $(PRERELEASES):
	$(MKDIR_CMD)
	$(SET_RUSTFLAGS) cargo build --release $(CARGO_FLAGS) --features $(PACKAGE_FEATURES) -p valkyrie --bin $(BIN)
	$(COPY_CMD)
