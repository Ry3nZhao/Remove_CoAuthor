Name:           rmai
Version:        0.1.0
Release:        1%{?dist}
Summary:        Remove AI co-author markers from Git history
License:        MIT
URL:            https://github.com/Ry3nZhao/Remove_CoAuthor
Source0:        https://github.com/Ry3nZhao/Remove_CoAuthor/releases/download/v%{version}/rmai-x86_64-unknown-linux-gnu.tar.gz
BuildArch:      x86_64
Requires:       git

%description
rmai detects and removes AI co-author trailers and related attribution
markers from Git commit messages, annotated tags, and notes, without
changing file trees. Optional commit-msg hooks (no-more) and a global
identity whitelist are included.

%prep
%setup -q -n rmai-x86_64-unknown-linux-gnu

%install
mkdir -p %{buildroot}%{_bindir}
install -m 0755 rmai %{buildroot}%{_bindir}/rmai

%files
%{_bindir}/rmai
%doc README.md README.zh-CN.md
%license LICENSE

%changelog
* Sun Nov 02 2026 rmai maintainers - 0.1.0-1
- Initial package
