#!/usr/bin/env python3
"""
GraphDB E2E Test Runner

Main entry point for running all E2E tests.
Provides options for running specific test suites and generating reports.

Usage:
    python run_tests.py                          # Run all tests
    python run_tests.py --suite social           # Run social network tests only
    python run_tests.py --suite optimizer        # Run optimizer tests only
    python run_tests.py --suite extended         # Run extended types tests only
    python run_tests.py --generate-data          # Generate test data only
    python run_tests.py --report junit           # Generate JUnit XML report
"""

import argparse
import sys
import time
import json
import os
from pathlib import Path
from datetime import datetime
from typing import List, Dict, Any, Optional
import unittest

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent))

from graphdb_client import GraphDBClient


class TestRunner:
    """E2E test runner with reporting capabilities."""

    def __init__(self, host: str = "127.0.0.1", port: int = 9758):
        self.host = host
        self.port = port
        self.client = GraphDBClient(host=host, port=port)
        self.results: Dict[str, Any] = {
            "start_time": None,
            "end_time": None,
            "duration_seconds": 0,
            "suites": [],
            "summary": {
                "total": 0,
                "passed": 0,
                "failed": 0,
                "errors": 0,
                "skipped": 0
            }
        }

    def check_server(self) -> bool:
        """Check if GraphDB server is running."""
        print(f"Checking GraphDB server at {self.host}:{self.port}...")
        if self.client.connect():
            print("✓ Server is ready")
            return True
        else:
            print("✗ Server is not available")
            print(f"  Please start GraphDB server first:")
            print(f"    cargo run --release")
            return False

    def generate_test_data(self, output_dir: str = "data"):
        """Generate test data using the generator script."""
        print("\nGenerating test data...")

        script_path = Path(__file__).parent.parent.parent / "scripts" / "generate_e2e_data.py"

        if not script_path.exists():
            print(f"Error: Generator script not found at {script_path}")
            return False

        import subprocess

        output_path = Path(__file__).parent / output_dir
        output_path.mkdir(exist_ok=True)

        result = subprocess.run(
            [sys.executable, str(script_path), "--output-dir", str(output_path)],
            capture_output=True,
            text=True
        )

        if result.returncode == 0:
            print("✓ Test data generated successfully")
            print(f"  Output directory: {output_path}")
            return True
        else:
            print("✗ Failed to generate test data")
            print(result.stderr)
            return False

    def run_suite(self, suite_name: str, test_classes: List[type]) -> bool:
        """Run a test suite."""
        print(f"\n{'='*60}")
        print(f"Running Test Suite: {suite_name}")
        print('='*60)

        loader = unittest.TestLoader()
        suite = unittest.TestSuite()

        for test_class in test_classes:
            suite.addTests(loader.loadTestsFromTestCase(test_class))

        runner = unittest.TextTestRunner(verbosity=2)
        result = runner.run(suite)

        suite_result = {
            "name": suite_name,
            "total": result.testsRun,
            "passed": result.testsRun - len(result.failures) - len(result.errors) - len(result.skipped),
            "failed": len(result.failures),
            "errors": len(result.errors),
            "skipped": len(result.skipped),
            "success": result.wasSuccessful()
        }

        self.results["suites"].append(suite_result)
        self.results["summary"]["total"] += suite_result["total"]
        self.results["summary"]["passed"] += suite_result["passed"]
        self.results["summary"]["failed"] += suite_result["failed"]
        self.results["summary"]["errors"] += suite_result["errors"]
        self.results["summary"]["skipped"] += suite_result["skipped"]

        return result.wasSuccessful()

    def run_all_tests(self) -> bool:
        """Run all test suites."""
        self.results["start_time"] = datetime.now().isoformat()
        start = time.time()

        all_passed = True

        # First run schema manager initialization tests
        # These tests verify the core functionality works regardless of vector search config
        try:
            from test_schema_manager_init import (
                TestSchemaManagerInitialization,
                TestSchemaManagerErrorHandling
            )

            schema_classes = [
                TestSchemaManagerInitialization,
                TestSchemaManagerErrorHandling
            ]

            if not self.run_suite("Schema Manager Init", schema_classes):
                all_passed = False
                print("\n⚠ WARNING: Schema manager initialization tests failed!")
                print("   This indicates a server configuration issue.")
                print("   Subsequent tests may also fail.\n")

        except Exception as e:
            print(f"Error running schema manager tests: {e}")
            all_passed = False

        # Import test modules
        try:
            from test_social_network import (
                TestSocialNetworkBasic,
                TestSocialNetworkData,
                TestSocialNetworkQueries,
                TestSocialNetworkExplain,
                TestSocialNetworkTransaction,
                TestSocialNetworkCleanup
            )

            social_classes = [
                TestSocialNetworkBasic,
                TestSocialNetworkData,
                TestSocialNetworkQueries,
                TestSocialNetworkExplain,
                TestSocialNetworkTransaction,
                TestSocialNetworkCleanup
            ]

            if not self.run_suite("Social Network", social_classes):
                all_passed = False

        except Exception as e:
            print(f"Error running social network tests: {e}")
            all_passed = False

        try:
            from test_optimizer import (
                TestOptimizerIndex,
                TestOptimizerJoin,
                TestOptimizerAggregate,
                TestOptimizerTopN,
                TestOptimizerExplainFormat,
                TestOptimizerProfile,
                TestOptimizerCleanup
            )

            optimizer_classes = [
                TestOptimizerIndex,
                TestOptimizerJoin,
                TestOptimizerAggregate,
                TestOptimizerTopN,
                TestOptimizerExplainFormat,
                TestOptimizerProfile,
                TestOptimizerCleanup
            ]

            if not self.run_suite("Optimizer", optimizer_classes):
                all_passed = False

        except Exception as e:
            print(f"Error running optimizer tests: {e}")
            all_passed = False

        try:
            from test_extended_types import (
                TestGeography,
                TestVector,
                TestFullText,
                TestExtendedTypesCleanup
            )

            extended_classes = [
                TestGeography,
                TestVector,
                TestFullText,
                TestExtendedTypesCleanup
            ]

            if not self.run_suite("Extended Types", extended_classes):
                all_passed = False

        except Exception as e:
            print(f"Error running extended types tests: {e}")
            all_passed = False

        self.results["end_time"] = datetime.now().isoformat()
        self.results["duration_seconds"] = round(time.time() - start, 2)

        return all_passed

    def print_summary(self):
        """Print test summary."""
        print("\n" + "="*60)
        print("TEST SUMMARY")
        print("="*60)

        for suite in self.results["suites"]:
            status = "✓ PASS" if suite["success"] else "✗ FAIL"
            print(f"\n{status} - {suite['name']}")
            print(f"  Total: {suite['total']}")
            print(f"  Passed: {suite['passed']}")
            print(f"  Failed: {suite['failed']}")
            print(f"  Errors: {suite['errors']}")
            print(f"  Skipped: {suite['skipped']}")

        print("\n" + "-"*60)
        print("OVERALL")
        print("-"*60)
        summary = self.results["summary"]
        print(f"Total Tests: {summary['total']}")
        print(f"Passed: {summary['passed']}")
        print(f"Failed: {summary['failed']}")
        print(f"Errors: {summary['errors']}")
        print(f"Skipped: {summary['skipped']}")
        print(f"Duration: {self.results['duration_seconds']:.2f}s")

        if summary['failed'] > 0 or summary['errors'] > 0:
            print("\n✗ SOME TESTS FAILED")
        else:
            print("\n✓ ALL TESTS PASSED")

    def save_report(self, filepath: str, format_type: str = "json"):
        """Save test report to file."""
        if format_type == "json":
            with open(filepath, 'w') as f:
                json.dump(self.results, f, indent=2)
            print(f"\nJSON report saved to: {filepath}")

        elif format_type == "junit":
            self._save_junit_report(filepath)
            print(f"\nJUnit XML report saved to: {filepath}")

    def _save_junit_report(self, filepath: str):
        """Save JUnit XML format report."""
        import xml.etree.ElementTree as ET

        testsuites = ET.Element("testsuites")
        testsuites.set("time", str(self.results["duration_seconds"]))
        testsuites.set("tests", str(self.results["summary"]["total"]))
        testsuites.set("failures", str(self.results["summary"]["failed"]))
        testsuites.set("errors", str(self.results["summary"]["errors"]))

        for suite_data in self.results["suites"]:
            testsuite = ET.SubElement(testsuites, "testsuite")
            testsuite.set("name", suite_data["name"])
            testsuite.set("tests", str(suite_data["total"]))
            testsuite.set("failures", str(suite_data["failed"]))
            testsuite.set("errors", str(suite_data["errors"]))
            testsuite.set("skipped", str(suite_data["skipped"]))

        tree = ET.ElementTree(testsuites)
        tree.write(filepath, encoding="utf-8", xml_declaration=True)


def main():
    parser = argparse.ArgumentParser(
        description="GraphDB E2E Test Runner"
    )
    parser.add_argument(
        "--suite",
        type=str,
        choices=["social", "optimizer", "extended", "all"],
        default="all",
        help="Test suite to run"
    )
    parser.add_argument(
        "--generate-data",
        action="store_true",
        help="Generate test data before running tests"
    )
    parser.add_argument(
        "--report",
        type=str,
        choices=["json", "junit"],
        help="Generate test report in specified format"
    )
    parser.add_argument(
        "--report-file",
        type=str,
        default="e2e_test_report",
        help="Report file name (without extension)"
    )
    parser.add_argument(
        "--host",
        type=str,
        default="127.0.0.1",
        help="GraphDB server host"
    )
    parser.add_argument(
        "--port",
        type=int,
        default=9758,
        help="GraphDB server port"
    )

    args = parser.parse_args()

    runner = TestRunner(host=args.host, port=args.port)

    # Check server
    if not runner.check_server():
        return 1

    # Generate test data if requested
    if args.generate_data:
        if not runner.generate_test_data():
            return 1

    # Run tests
    success = runner.run_all_tests()

    # Print summary
    runner.print_summary()

    # Save report if requested
    if args.report:
        report_dir = Path(__file__).parent / "reports"
        report_dir.mkdir(exist_ok=True)

        if args.report == "json":
            report_file = report_dir / f"{args.report_file}.json"
        else:
            report_file = report_dir / f"{args.report_file}.xml"

        runner.save_report(str(report_file), args.report)

    return 0 if success else 1


if __name__ == "__main__":
    sys.exit(main())
