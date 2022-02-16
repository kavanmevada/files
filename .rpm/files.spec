%define __spec_install_post %{nil}
%define __os_install_post %{_dbpath}/brp-compress
%define debug_package %{nil}

Name: files
Summary: Access and organize files
Version: @@VERSION@@
Release: @@RELEASE@@%{?dist}
License: MIT or ASL 2.0
Group: Applications/System
Source0: %{name}-%{version}.tar.gz
URL: https://example.com/

Requires: gtk4-devel, libadwaita-devel
BuildRoot: %{_tmppath}/%{name}-%{version}-%{release}-root

%description
%{summary}

%prep
%setup -q

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
cp -a * %{buildroot}
install -m 755 -D org.kavanmevada.Files.svg -t %{buildroot}/usr/share/icons/hicolor/scalable/apps
install -m 755 -D org.kavanmevada.Files.desktop -t %{buildroot}/usr/share/applications
install -m 755 -D org.kavanmevada.Files.metainfo.xml -t %{buildroot}/usr/share/metainfo


%clean
rm -rf %{buildroot}

%files
%defattr(-,root,root,-)
%{_bindir}/*
/usr/share/icons/hicolor/scalable/apps/org.kavanmevada.Files.svg
/usr/share/applications/org.kavanmevada.Files.desktop
/usr/share/metainfo/org.kavanmevada.Files.metainfo.xml