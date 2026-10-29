import importlib.util
from pathlib import Path
import sys
import tempfile
import unittest
import xml.etree.ElementTree as ET


ROOT = Path(__file__).resolve().parents[1]


class ModelDescriptionVersionTests(unittest.TestCase):
    def test_regenerated_xml_identifies_generator_version(self):
        template = (ROOT / "assets/python/fmi2/fmi2_model_description.py").read_text(
            encoding="utf-8"
        )
        for version in ["0.14.0", "9.8.7"]:
            with self.subTest(version=version), tempfile.TemporaryDirectory() as directory:
                helper = Path(directory) / "fmi2_model_description.py"
                helper.write_text(
                    template.replace("__UNIFMU_VERSION__", version), encoding="utf-8"
                )
                spec = importlib.util.spec_from_file_location("versioned_model_description", helper)
                module = importlib.util.module_from_spec(spec)
                sys.modules[spec.name] = module
                try:
                    spec.loader.exec_module(module)
                    model = type("Model", (), {})()
                    xml = module.generate_model_description_xml(model)
                    root = ET.fromstring(xml)
                    self.assertEqual(root.attrib["generationTool"], f"unifmu-{version}")
                    self.assertEqual(root.attrib["fmiVersion"], "2.0")
                    self.assertEqual(root.attrib["modelName"], "unifmu")
                finally:
                    del sys.modules[spec.name]


if __name__ == "__main__":
    unittest.main()
