%global debug_package %{nil}
%define _build_id_links none

Name:           mx-options
Version:        %{?!pkg_version:0.1.0}
Release:        %{?!pkg_release:1}%{?dist}
Summary:        Control y configuracion nativa para ratones Logitech MX en Linux
License:        GPL-2.0-only
URL:            https://github.com/Panxio/mxpilot
AutoReqProv:    no

Requires:       webkit2gtk4.1
Requires:       gtk3
Requires:       libayatana-appindicator-gtk3
Requires:       systemd
Requires:       systemd-udev
Requires:       procps-ng

%description
mx-options (MX Pilot) es una suite ligera y nativa para configurar ratones
Logitech MX (especialmente MX Vertical) bajo Linux, sin permisos de root en
tiempo de ejecucion. Incluye el servicio de sesion (mx-daemon), la interfaz
grafica (mx-gui), la CLI (mx-cli), la sonda de protocolo (mx-probe) y
telemetria (mx-diag).

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
if [ -n "%{?staging_dir}" ] && [ -d "%{?staging_dir}" ]; then
    cp -a %{staging_dir}/* %{buildroot}/
fi

%post
if [ -f /etc/udev/rules.d/99-mx-options.rules ]; then
    if grep -q "MX Options Linux" /etc/udev/rules.d/99-mx-options.rules 2>/dev/null; then
        rm -f /etc/udev/rules.d/99-mx-options.rules
    fi
fi
if [ -f /etc/udev/rules.d/70-mx-options.rules ]; then
    if grep -q "MX Options Linux" /etc/udev/rules.d/70-mx-options.rules 2>/dev/null; then
        rm -f /etc/udev/rules.d/70-mx-options.rules
    fi
fi
if [ -d /run/udev ]; then
    udevadm control --reload-rules 2>/dev/null || true
    udevadm trigger --subsystem-match=hidraw --subsystem-match=misc 2>/dev/null || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor 2>/dev/null || true
fi
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database -q /usr/share/applications 2>/dev/null || true
fi
exit 0

%preun
if [ "$1" = "0" ]; then
    if command -v pkill >/dev/null 2>&1; then
        if pkill -x -TERM mx-daemon 2>/dev/null; then
            i=0
            while pkill -x -0 mx-daemon 2>/dev/null && [ "$i" -lt 25 ]; do
                sleep 0.1
                i=$((i + 1))
            done
            if pkill -x -0 mx-daemon 2>/dev/null; then
                pkill -x -KILL mx-daemon 2>/dev/null || true
            fi
        fi
    fi
fi
exit 0

%postun
if [ "$1" = "0" ]; then
    if [ -d /run/udev ]; then
        udevadm control --reload-rules 2>/dev/null || true
        udevadm trigger --subsystem-match=hidraw --subsystem-match=misc 2>/dev/null || true
    fi
    if command -v gtk-update-icon-cache >/dev/null 2>&1; then
        gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor 2>/dev/null || true
    fi
    if command -v update-desktop-database >/dev/null 2>&1; then
        update-desktop-database -q /usr/share/applications 2>/dev/null || true
    fi
fi
exit 0

%files
%defattr(0644,root,root,0755)
%attr(0755,root,root) /usr/bin/mx-daemon
%attr(0755,root,root) /usr/bin/mx-gui
%attr(0755,root,root) /usr/bin/mx-diag
%attr(0755,root,root) /usr/bin/mx-cli
%attr(0755,root,root) /usr/bin/mx-probe
/usr/lib/udev/rules.d/70-mx-options.rules
/usr/lib/systemd/user/mx-options.service
/usr/share/applications/mx-options.desktop
/usr/share/doc/mx-options/LICENSE
/usr/share/doc/mx-options/copyright
/usr/share/icons/hicolor/*/apps/mx-options.png
/usr/share/icons/hicolor/scalable/apps/mx-options.svg
