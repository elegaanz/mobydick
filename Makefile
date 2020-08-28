TARGET ?= target/release/mobydick
PREFIX ?= /usr/local
CARGO ?= cargo

UNINSTALL_FILES = $(PREFIX)/bin/xyz.gelez.mobydick \
	$(PREFIX)/share/appdata/xyz.gelez.mobydick.appdata.xml \
	$(PREFIX)/share/applications/mobydick.desktop \
	$(PREFIX)/share/icons/hicolor/128x128/apps/mobydick.svg \
	$(PREFIX)/share/icons/hicolor/16x16/apps/mobydick.svg \
	$(PREFIX)/share/icons/hicolor/24x24/apps/mobydick.svg \
	$(PREFIX)/share/icons/hicolor/32x32/apps/mobydick.svg \
	$(PREFIX)/share/icons/hicolor/48x48/apps/mobydick.svg \
	$(PREFIX)/share/icons/hicolor/64x64/apps/mobydick.svg

.phony: all
all: $(TARGET)

$(TARGET):
	$(CARGO) build --release

.phony: check
chuck:
	$(CARGO) check && $(CARGO) test

.phony: install
install: install-exe install-data

.phony: install-exe
install-exe:
	install -d $(PREFIX)/bin/ && \
	install target/release/mobydick $(PREFIX)/bin/xyz.gelez.mobydick

.phony: install-data
install-data: install-app-data install-app-desktop install-icons

.phony: install-app-data
install-app-data:
	install -d $(PREFIX)/share/appdata && \
	install -m -x -t $(PREFIX)/share/appdata xyz.gelez.mobydick.appdata.xml

.phony: install-app-desktop
install-app-desktop:
	install -d $(PREFIX)/share/applications && \
	install -m -x -t $(PREFIX)/share/applications mobydick.desktop

.phony: install-icons
install-icons:
	for s in "16" "24" "32" "48" "64" "128"; do \
  		install -d $(PREFIX)/share/icons/hicolor/$${s}x$${s}/apps/ && \
  		install -m -x icons/$$s.svg $(PREFIX)/share/icons/hicolor/$${s}x$${s}/apps/mobydick.svg ;\
	done

.phony: uninstall
uninstall:
	$(RM) -f $(UNINSTALL_FILES)

.phony: clean
clean:
	$(CARGO) clean