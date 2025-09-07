#!/bin/bash
# Alpenglow Timeout Handling Verification Script
# This script verifies timeout mechanisms and skip certificate generation

set -e

echo "=== Alpenglow Timeout Handling Verification ==="
echo "Timestamp: $(date)"
echo "Testing timeout mechanisms and skip certificate generation"
echo ""

# Create results directory
mkdir -p results/timeout_verification
cd results/timeout_verification

# Initialize results file
echo "Test_Type,Timeout_MS,Test_Run,Result,Details" > timeout_results.csv

# Test 1: Basic timeout handling
echo "Test 1: Basic timeout handling"

for timeout_ms in 50 100 150 200 300; do
    echo "Testing with $timeout_ms ms timeout"
    
    for i in $(seq 1 30); do
        cargo run --release --bin timeout_verification -- --test-type basic --timeout $timeout_ms --seed $i > basic_${timeout_ms}ms_${i}.log 2>&1
        
        if grep -q "Timeout handling successful" basic_${timeout_ms}ms_${i}.log; then
            echo "basic,$timeout_ms,$i,SUCCESS,Valid timeout handling" >> timeout_results.csv
        else
            echo "basic,$timeout_ms,$i,FAILURE,Invalid timeout handling" >> timeout_results.csv
        fi
    done
done

# Test 2: Skip certificate generation
echo "Test 2: Skip certificate generation"

for i in $(seq 1 50); do
    cargo run --release --bin timeout_verification -- --test-type skip_cert --seed $i > skip_cert_${i}.log 2>&1
    
    if grep -q "Skip certificate generation successful" skip_cert_${i}.log; then
        echo "skip_cert,100,$i,SUCCESS,Valid skip certificate" >> timeout_results.csv
    else
        echo "skip_cert,100,$i,FAILURE,Invalid skip certificate" >> timeout_results.csv
    fi
done

# Test 3: BadWindow flag triggering
echo "Test 3: BadWindow flag triggering"

for i in $(seq 1 50); do
    cargo run --release --bin timeout_verification -- --test-type badwindow --seed $i > badwindow_${i}.log 2>&1
    
    if grep -q "BadWindow flag triggered correctly" badwindow_${i}.log; then
        echo "badwindow,100,$i,SUCCESS,Valid BadWindow triggering" >> timeout_results.csv
    else
        echo "badwindow,100,$i,FAILURE,Invalid BadWindow triggering" >> timeout_results.csv
    fi
done

# Test 4: Timeout with different network conditions
echo "Test 4: Timeout with different network conditions"

for network_delay in 10 20 30 40 50; do
    echo "Testing with $network_delay ms network delay"
    
    for i in $(seq 1 30); do
        cargo run --release --bin timeout_verification -- --test-type network_delay --delay $network_delay --seed $i > network_${network_delay}ms_${i}.log 2>&1
        
        if grep -q "Network delay handling successful" network_${network_delay}ms_${i}.log; then
            echo "network_delay,$network_delay,$i,SUCCESS,Valid network delay handling" >> timeout_results.csv
        else
            echo "network_delay,$network_delay,$i,FAILURE,Invalid network delay handling" >> timeout_results.csv
        fi
    done
done

# Test 5: Timeout recovery
echo "Test 5: Timeout recovery"

for i in $(seq 1 50); do
    cargo run --release --bin timeout_verification -- --test-type recovery --seed $i > recovery_${i}.log 2>&1
    
    if grep -q "Timeout recovery successful" recovery_${i}.log; then
        echo "recovery,100,$i,SUCCESS,Valid timeout recovery" >> timeout_results.csv
    else
        echo "recovery,100,$i,FAILURE,Invalid timeout recovery" >> timeout_results.csv
    fi
done

# Test 6: Concurrent timeouts
echo "Test 6: Concurrent timeouts"

for i in $(seq 1 30); do
    cargo run --release --bin timeout_verification -- --test-type concurrent --seed $i > concurrent_${i}.log 2>&1
    
    if grep -q "Concurrent timeout handling successful" concurrent_${i}.log; then
        echo "concurrent,100,$i,SUCCESS,Valid concurrent timeout handling" >> timeout_results.csv
    else
        echo "concurrent,100,$i,FAILURE,Invalid concurrent timeout handling" >> timeout_results.csv
    fi
done

# Test 7: Timeout with partial network
echo "Test 7: Timeout with partial network"

for offline_percent in 10 20 30 40; do
    echo "Testing with $offline_percent% offline nodes"
    
    for i in $(seq 1 30); do
        cargo run --release --bin timeout_verification -- --test-type partial_network --offline-percent $offline_percent --seed $i > partial_${offline_percent}pct_${i}.log 2>&1
        
        if grep -q "Partial network timeout handling successful" partial_${offline_percent}pct_${i}.log; then
            echo "partial_network,$offline_percent,$i,SUCCESS,Valid partial network handling" >> timeout_results.csv
        else
            echo "partial_network,$offline_percent,$i,FAILURE,Invalid partial network handling" >> timeout_results.csv
        fi
    done
done

# Generate summary report
echo ""
echo "=== Timeout Handling Verification Summary ==="

# Analyze results by test type
echo "Results by Test Type:"
for test_type in basic skip_cert badwindow network_delay recovery concurrent partial_network; do
    count=$(grep "^$test_type," timeout_results.csv | wc -l)
    success=$(grep "^$test_type," timeout_results.csv | grep -c "SUCCESS" || true)
    
    if [ $count -gt 0 ]; then
        success_rate=$((success * 100 / count))
        echo "  $test_type: $success_rate% success rate ($success/$count)"
    fi
done

# Analyze timeout performance
echo ""
echo "Timeout Performance:"
for timeout_ms in 50 100 150 200 300; do
    count=$(grep "basic,$timeout_ms," timeout_results.csv | wc -l)
    success=$(grep "basic,$timeout_ms," timeout_results.csv | grep -c "SUCCESS" || true)
    
    if [ $count -gt 0 ]; then
        success_rate=$((success * 100 / count))
        echo "  $timeout_ms ms timeout: $success_rate% success rate ($success/$count)"
    fi
done

# Analyze network delay performance
echo ""
echo "Network Delay Performance:"
for network_delay in 10 20 30 40 50; do
    count=$(grep "network_delay,$network_delay," timeout_results.csv | wc -l)
    success=$(grep "network_delay,$network_delay," timeout_results.csv | grep -c "SUCCESS" || true)
    
    if [ $count -gt 0 ]; then
        success_rate=$((success * 100 / count))
        echo "  $network_delay ms delay: $success_rate% success rate ($success/$count)"
    fi
done

# Analyze partial network performance
echo ""
echo "Partial Network Performance:"
for offline_percent in 10 20 30 40; do
    count=$(grep "partial_network,$offline_percent," timeout_results.csv | wc -l)
    success=$(grep "partial_network,$offline_percent," timeout_results.csv | grep -c "SUCCESS" || true)
    
    if [ $count -gt 0 ]; then
        success_rate=$((success * 100 / count))
        echo "  $offline_percent% offline: $success_rate% success rate ($success/$count)"
    fi
done

# Verify critical properties
echo ""
echo "=== Critical Property Verification ==="

# Check if basic timeout handling works
basic_success=$(grep "^basic," timeout_results.csv | grep -c "SUCCESS" || true)
basic_total=$(grep "^basic," timeout_results.csv | wc -l)
basic_rate=$((basic_success * 100 / basic_total))

if [ $basic_rate -ge 90 ]; then
    echo "✅ Basic timeout handling works correctly ($basic_rate%)"
else
    echo "❌ Basic timeout handling has issues ($basic_rate%)"
fi

# Check if skip certificate generation works
skip_success=$(grep "^skip_cert," timeout_results.csv | grep -c "SUCCESS" || true)
skip_rate=$((skip_success * 100 / 50))

if [ $skip_rate -ge 90 ]; then
    echo "✅ Skip certificate generation works correctly ($skip_rate%)"
else
    echo "❌ Skip certificate generation has issues ($skip_rate%)"
fi

# Check if BadWindow flag triggering works
badwindow_success=$(grep "^badwindow," timeout_results.csv | grep -c "SUCCESS" || true)
badwindow_rate=$((badwindow_success * 100 / 50))

if [ $badwindow_rate -ge 90 ]; then
    echo "✅ BadWindow flag triggering works correctly ($badwindow_rate%)"
else
    echo "❌ BadWindow flag triggering has issues ($badwindow_rate%)"
fi

# Check if timeout recovery works
recovery_success=$(grep "^recovery," timeout_results.csv | grep -c "SUCCESS" || true)
recovery_rate=$((recovery_success * 100 / 50))

if [ $recovery_rate -ge 90 ]; then
    echo "✅ Timeout recovery works correctly ($recovery_rate%)"
else
    echo "❌ Timeout recovery has issues ($recovery_rate%)"
fi

# Check if concurrent timeout handling works
concurrent_success=$(grep "^concurrent," timeout_results.csv | grep -c "SUCCESS" || true)
concurrent_rate=$((concurrent_success * 100 / 30))

if [ $concurrent_rate -ge 80 ]; then
    echo "✅ Concurrent timeout handling works correctly ($concurrent_rate%)"
else
    echo "❌ Concurrent timeout handling has issues ($concurrent_rate%)"
fi

# Check if partial network handling works
partial_success=$(grep "^partial_network," timeout_results.csv | grep -c "SUCCESS" || true)
partial_total=$(grep "^partial_network," timeout_results.csv | wc -l)
partial_rate=$((partial_success * 100 / partial_total))

if [ $partial_rate -ge 80 ]; then
    echo "✅ Partial network timeout handling works correctly ($partial_rate%)"
else
    echo "❌ Partial network timeout handling has issues ($partial_rate%)"
fi

# Generate performance analysis
echo ""
echo "=== Performance Analysis ==="

# Analyze timeout efficiency
echo "Timeout Efficiency:"
for timeout_ms in 50 100 150 200 300; do
    avg_time=$(grep "basic,$timeout_ms,.*,SUCCESS" timeout_results.csv | awk -F',' '{sum+=$5; count++} END {if(count>0) print sum/count; else print "N/A"}')
    echo "  $timeout_ms ms timeout: avg response time $avg_time"
done

# Analyze network delay impact
echo ""
echo "Network Delay Impact:"
for network_delay in 10 20 30 40 50; do
    avg_time=$(grep "network_delay,$network_delay,.*,SUCCESS" timeout_results.csv | awk -F',' '{sum+=$5; count++} END {if(count>0) print sum/count; else print "N/A"}')
    echo "  $network_delay ms delay: avg response time $avg_time"
done

# Analyze partial network impact
echo ""
echo "Partial Network Impact:"
for offline_percent in 10 20 30 40; do
    avg_time=$(grep "partial_network,$offline_percent,.*,SUCCESS" timeout_results.csv | awk -F',' '{sum+=$5; count++} END {if(count>0) print sum/count; else print "N/A"}')
    echo "  $offline_percent% offline: avg response time $avg_time"
done

# Generate recommendations
echo ""
echo "=== Recommendations ==="

# Check overall success rate
total_tests=$(wc -l < timeout_results.csv)
successful_tests=$(grep -c "SUCCESS" timeout_results.csv || true)
overall_rate=$((successful_tests * 100 / total_tests))

if [ $overall_rate -ge 90 ]; then
    echo "✅ Overall timeout handling verification successful ($overall_rate%)"
    echo "✅ Timeout mechanisms and skip certificate generation are working correctly"
    echo "✅ The system can handle various timeout scenarios and network conditions"
else
    echo "⚠️  Overall timeout handling verification needs improvement ($overall_rate%)"
    echo "⚠️  Some aspects of timeout handling may need attention"
fi

# Specific recommendations based on results
if [ $basic_rate -lt 90 ]; then
    echo "⚠️  Consider reviewing basic timeout handling logic"
fi

if [ $skip_rate -lt 90 ]; then
    echo "⚠️  Consider reviewing skip certificate generation"
fi

if [ $badwindow_rate -lt 90 ]; then
    echo "⚠️  Consider reviewing BadWindow flag triggering logic"
fi

if [ $recovery_rate -lt 90 ]; then
    echo "⚠️  Consider improving timeout recovery mechanisms"
fi

if [ $concurrent_rate -lt 80 ]; then
    echo "⚠️  Consider improving concurrent timeout handling"
fi

if [ $partial_rate -lt 80 ]; then
    echo "⚠️  Consider improving partial network timeout handling"
fi

echo ""
echo "Timeout handling verification completed. Check timeout_results.csv for detailed results."
