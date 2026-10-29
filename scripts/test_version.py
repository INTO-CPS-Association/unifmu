from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import tomllib
import unittest

from version import ROOT, parse_version, select_version, set_version


class VersionTests(unittest.TestCase):
    def test_next_patch(self):
        self.assertEqual(select_version("0.14.0", ["0.0.11", "v0.14.0", "v0.15.0-beta"], None), "0.14.1")
        self.assertEqual(select_version("1.2.3", ["v1.2.3"], None), "1.2.4")
        self.assertEqual(select_version("1.2.3", ["v1.4.0"], None), "1.4.1")
        self.assertEqual(select_version("1.2.3", [], None), "1.2.4")

    def test_override(self):
        self.assertEqual(select_version("0.14.0", ["v0.14.0"], "v0.14.1"), "0.14.1")
        self.assertEqual(select_version("0.15.0", ["v0.14.0"], "0.15.0"), "0.15.0")
        self.assertEqual(select_version("0.14.0", [], "0.14.0"), "0.14.0")

    def test_reject_invalid_or_existing_version(self):
        for value in ["0.14.0", "0.13.1", "0.15.0-beta", "01.15.0", "0.15", "v0.15.0\nbad"]:
            with self.subTest(value=value), self.assertRaises(ValueError):
                select_version("0.14.0", ["v0.14.0"], value)

    def test_parse_version(self):
        self.assertEqual(parse_version("v1.2.3"), (1, 2, 3))

    def test_update_only_workspace_version(self):
        manifest = '[workspace]\nmembers = []\n\n[workspace.package]\nversion = "0.14.0"\n\n[other]\nversion = "2.0.0"\n'
        updated = set_version(manifest, "0.15.0")
        self.assertIn('[workspace.package]\nversion = "0.15.0"', updated)
        self.assertIn('[other]\nversion = "2.0.0"', updated)
        with self.assertRaises(ValueError):
            set_version("[workspace]\n", "0.15.0")

    def test_selected_version_is_written_as_valid_toml(self):
        manifest = '[workspace.package]\nversion = "0.14.0"\n'
        version = select_version("0.14.0", ["v0.14.0"], None)
        updated = tomllib.loads(set_version(manifest, version))
        self.assertEqual(updated["workspace"]["package"]["version"], "0.14.1")

    def test_all_crates_inherit_the_workspace_version(self):
        manifest = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))
        parse_version(manifest["workspace"]["package"]["version"])
        for member in manifest["workspace"]["members"]:
            with self.subTest(member=member):
                package = tomllib.loads((ROOT / member / "Cargo.toml").read_text(encoding="utf-8"))
                self.assertEqual(package["package"]["version"], {"workspace": True})

    def test_prepare_command_persists_version_and_rejects_bad_input(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "scripts").mkdir()
            script = root / "scripts" / "version.py"
            shutil.copyfile(ROOT / "scripts" / "version.py", script)
            manifest = root / "Cargo.toml"
            manifest.write_text('[workspace.package]\nversion = "0.14.0"\n', encoding="utf-8")
            subprocess.run(["git", "init", "--quiet", str(root)], check=True)
            result = subprocess.run(
                [sys.executable, str(script), "--prepare"],
                check=True, capture_output=True, text=True,
            )
            self.assertEqual(result.stdout.strip(), "0.14.1")
            self.assertEqual(tomllib.loads(manifest.read_text())["workspace"]["package"]["version"], "0.14.1")
            prepared = manifest.read_text()
            result = subprocess.run(
                [sys.executable, str(script), "--prepare", "--version", "bad"],
                capture_output=True, text=True,
            )
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("Expected a stable version", result.stderr)
            self.assertEqual(manifest.read_text(), prepared)


if __name__ == "__main__":
    unittest.main()
