%define __spec_install_post %{nil}
%define __os_install_post %{_dbpath}/brp-compress
%define debug_package %{nil}

Name: terra
Summary: Front page of Skyland-infused World of Warcraft servers
Version: @@VERSION@@
Release: @@RELEASE@@
License: Proprietary
Group: System Environment/Daemons
Source0: %{name}-%{version}.tar.gz

BuildRoot: %{_tmppath}/%{name}-%{version}-%{release}-root
BuildRequires: systemd

Requires(post): systemd
Requires(preun): systemd
Requires(postun): systemd

%description
%{summary}

%prep
%setup -q

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
cp -a * %{buildroot}

%clean
rm -rf %{buildroot}

%systemd_post terra.service

%preun
%systemd_preun terra.service

%postun
%systemd_postun_with_restart terra.service

%files
%defattr(-,root,root,-)
%{_sbindir}/*
%{_unitdir}/terra.service
