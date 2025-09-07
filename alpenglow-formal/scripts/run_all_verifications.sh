#!/bin/bash
# Complete Alpenglow Formal Verification Suite
# This script runs all verification tests and generates comprehensive reports

set -e

echo "=== Complete Alpenglow Formal Verification Suite ==="
echo "Starting comprehensive verification at $(date)"
echo ""

# Create timestamped results directory
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
mkdir -p results/$TIMESTAMP
cd results/$TIMESTAMP

# Initialize master results file
echo "Test_Suite,Test_Type,Configuration,Result,Details" > master_results.csv

# Function to run a verification script and capture results
run_verification() {
    local script_name=$1
    local test_type=$2
    local config=$3
    
    echo "Running $script_name..."
    
    if bash ../../scripts/$script_name; then
        echo "$script_name,$test_type,$config,SUCCESS,All tests passed" >> master_results.csv
        echo "✅ $script_name completed successfully"
    else
        echo "$script_name,$test_type,$config,FAILURE,Some tests failed" >> master_results.csv
        echo "❌ $script_name had failures"
    fi
    
    echo ""
}

# 1. Safety Property Verification
echo "=== 1. Safety Property Verification ==="
run_verification "verify_safety_properties.sh" "safety" "2-5_validators_1-4_slots"

# 2. Liveness Property Verification
echo "=== 2. Liveness Property Verification ==="
run_verification "verify_liveness_properties.sh" "liveness" "50-90%_responsive_stake"

# 3. Resilience Property Verification
echo "=== 3. Resilience Property Verification ==="
run_verification "verify_resilience_properties.sh" "resilience" "byzantine_offline_partition_tests"

# 4. Certificate Uniqueness Verification
echo "=== 4. Certificate Uniqueness Verification ==="
run_verification "verify_certificate_uniqueness.sh" "certificate" "adversary_attack_simulation"

# 5. Leader Rotation Verification
echo "=== 5. Leader Rotation Verification ==="
run_verification "verify_leader_rotation.sh" "leader" "window_management_tests"

# 6. Timeout Handling Verification
echo "=== 6. Timeout Handling Verification ==="
run_verification "verify_timeout_handling.sh" "timeout" "skip_certificate_tests"

# 7. Rotor Sampling Verification
echo "=== 7. Rotor Sampling Verification ==="
run_verification "verify_rotor_sampling.sh" "rotor" "message_dissemination_tests"

# 8. Performance Benchmarking
echo "=== 8. Performance Benchmarking ==="
run_verification "benchmark_verification.sh" "performance" "scalability_tests"

# 9. Dual-Path Finality Verification
echo "=== 9. Dual-Path Finality Verification ==="
run_verification "verify_dual_path_finality.sh" "dual_path" "fast_vs_slow_path_tests"

# 10. Bounded Finalization Time Verification
echo "=== 10. Bounded Finalization Time Verification ==="
run_verification "verify_bounded_finalization.sh" "bounded_time" "timing_analysis_tests"

# Generate comprehensive summary report
echo "=== Generating Comprehensive Summary Report ==="

# Count results
total_tests=$(wc -l < master_results.csv)
successful_tests=$(grep -c "SUCCESS" master_results.csv || true)
failed_tests=$(grep -c "FAILURE" master_results.csv || true)

# Generate summary
cat > verification_summary.md << EOF
# Alpenglow Formal Verification Summary

**Verification Date:** $(date)
**Total Test Suites:** $((total_tests - 1))
**Successful:** $successful_tests
**Failed:** $failed_tests
**Success Rate:** $((successful_tests * 100 / (total_tests - 1)))%

## Test Results by Category

EOF

# Generate detailed results by category
for category in safety liveness resilience certificate leader timeout rotor performance dual_path bounded_time; do
    count=$(grep ",$category," master_results.csv | wc -l)
    success=$(grep ",$category,.*,SUCCESS" master_results.csv | wc -l)
    failure=$(grep ",$category,.*,FAILURE" master_results.csv | wc -l)
    
    if [ $count -gt 0 ]; then
        success_rate=$((success * 100 / count))
        echo "### $category" >> verification_summary.md
        echo "- Tests: $count" >> verification_summary.md
        echo "- Success: $success" >> verification_summary.md
        echo "- Failure: $failure" >> verification_summary.md
        echo "- Success Rate: $success_rate%" >> verification_summary.md
        echo "" >> verification_summary.md
    fi
done

# Add detailed results table
echo "## Detailed Results" >> verification_summary.md
echo "" >> verification_summary.md
echo "| Test Suite | Test Type | Configuration | Result | Details |" >> verification_summary.md
echo "|------------|-----------|---------------|--------|---------|" >> verification_summary.md

# Convert CSV to markdown table
tail -n +2 master_results.csv | while IFS=',' read -r suite type config result details; do
    echo "| $suite | $type | $config | $result | $details |" >> verification_summary.md
done

# Generate JSON report for programmatic access
cat > verification_results.json << EOF
{
  "verification_date": "$(date -Iseconds)",
  "total_tests": $((total_tests - 1)),
  "successful_tests": $successful_tests,
  "failed_tests": $failed_tests,
  "success_rate": $((successful_tests * 100 / (total_tests - 1))),
  "test_results": [
EOF

# Add individual test results to JSON
first=true
tail -n +2 master_results.csv | while IFS=',' read -r suite type config result details; do
    if [ "$first" = true ]; then
        first=false
    else
        echo "," >> verification_results.json
    fi
    echo "    {" >> verification_results.json
    echo "      \"test_suite\": \"$suite\"," >> verification_results.json
    echo "      \"test_type\": \"$type\"," >> verification_results.json
    echo "      \"configuration\": \"$config\"," >> verification_results.json
    echo "      \"result\": \"$result\"," >> verification_results.json
    echo "      \"details\": \"$details\"" >> verification_results.json
    echo -n "    }" >> verification_results.json
done

echo "" >> verification_results.json
echo "  ]" >> verification_results.json
echo "}" >> verification_results.json

# Generate final status
echo ""
echo "=== Verification Complete ==="
echo "All verifications completed at $(date)"
echo ""
echo "Results Summary:"
echo "- Total test suites: $((total_tests - 1))"
echo "- Successful: $successful_tests"
echo "- Failed: $failed_tests"
echo "- Success rate: $((successful_tests * 100 / (total_tests - 1)))%"
echo ""
echo "Generated reports:"
echo "- verification_summary.md: Human-readable summary"
echo "- verification_results.json: Machine-readable results"
echo "- master_results.csv: Detailed CSV results"
echo ""

# Check if all critical tests passed
critical_tests=("safety" "liveness" "resilience" "certificate")
all_critical_passed=true

for test_type in "${critical_tests[@]}"; do
    if ! grep -q ",$test_type,.*,SUCCESS" master_results.csv; then
        echo "❌ Critical test type '$test_type' failed"
        all_critical_passed=false
    fi
done

if [ "$all_critical_passed" = true ]; then
    echo "🎉 ALL CRITICAL VERIFICATION TESTS PASSED!"
    echo "The Alpenglow consensus protocol has been successfully verified."
    exit 0
else
    echo "⚠️  Some critical verification tests failed."
    echo "Please review the detailed results and address any issues."
    exit 1
fi
