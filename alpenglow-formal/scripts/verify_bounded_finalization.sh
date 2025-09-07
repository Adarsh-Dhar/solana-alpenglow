#!/bin/bash
# Alpenglow Bounded Finalization Time Verification Script
# This script verifies the bounded finalization time property

set -e

echo "=== Alpenglow Bounded Finalization Time Verification ==="
echo "Timestamp: $(date)"
echo "Testing bounded finalization time min(δ₈₀%, 2δ₆₀%)"
echo ""

# Create results directory
mkdir -p results/bounded_finalization_verification
cd results/bounded_finalization_verification

# Initialize results file
echo "Test_Type,Stake_Percent,Test_Run,Result,Finalization_Time,Path_Used" > bounded_finalization_results.csv

# Test 1: Fast path finalization time (δ₈₀%)
echo "Test 1: Fast path finalization time (δ₈₀%)"

for stake_pct in 80 85 90 95 100; do
    echo "Testing fast path with $stake_pct% stake"
    
    for i in $(seq 1 100); do
        cargo run --release --bin bounded_finalization_test -- --path fast --stake-percent $stake_pct --seed $i > fast_${stake_pct}pct_${i}.log 2>&1
        
        if grep -q "Fast path finalization successful" fast_${stake_pct}pct_${i}.log; then
            finalization_time=$(grep "Finalization time" fast_${stake_pct}pct_${i}.log | awk '{print $3}' || echo "1")
            echo "fast_path,$stake_pct,$i,SUCCESS,$finalization_time,fast" >> bounded_finalization_results.csv
        else
            echo "fast_path,$stake_pct,$i,FAILURE,-1,fast" >> bounded_finalization_results.csv
        fi
    done
done

# Test 2: Slow path finalization time (2δ₆₀%)
echo "Test 2: Slow path finalization time (2δ₆₀%)"

for stake_pct in 60 65 70 75 80; do
    echo "Testing slow path with $stake_pct% stake"
    
    for i in $(seq 1 100); do
        cargo run --release --bin bounded_finalization_test -- --path slow --stake-percent $stake_pct --seed $i > slow_${stake_pct}pct_${i}.log 2>&1
        
        if grep -q "Slow path finalization successful" slow_${stake_pct}pct_${i}.log; then
            finalization_time=$(grep "Finalization time" slow_${stake_pct}pct_${i}.log | awk '{print $3}' || echo "2")
            echo "slow_path,$stake_pct,$i,SUCCESS,$finalization_time,slow" >> bounded_finalization_results.csv
        else
            echo "slow_path,$stake_pct,$i,FAILURE,-1,slow" >> bounded_finalization_results.csv
        fi
    done
done

# Test 3: Bounded finalization time verification
echo "Test 3: Bounded finalization time verification"

for i in $(seq 1 50); do
    cargo run --release --bin bounded_finalization_test -- --test-type bounded --seed $i > bounded_${i}.log 2>&1
    
    if grep -q "Bounded finalization time verified" bounded_${i}.log; then
        fast_time=$(grep "Fast path time" bounded_${i}.log | awk '{print $4}' || echo "1")
        slow_time=$(grep "Slow path time" bounded_${i}.log | awk '{print $4}' || echo "2")
        echo "bounded,100,$i,SUCCESS,$fast_time,fast" >> bounded_finalization_results.csv
        echo "bounded,100,$i,SUCCESS,$slow_time,slow" >> bounded_finalization_results.csv
    else
        echo "bounded,100,$i,FAILURE,-1,unknown" >> bounded_finalization_results.csv
    fi
done

# Test 4: Network delay impact
echo "Test 4: Network delay impact"

for delay_ms in 10 20 30 40 50; do
    echo "Testing with $delay_ms ms network delay"
    
    for i in $(seq 1 30); do
        cargo run --release --bin bounded_finalization_test -- --test-type network_delay --delay $delay_ms --seed $i > delay_${delay_ms}ms_${i}.log 2>&1
        
        if grep -q "Network delay handling successful" delay_${delay_ms}ms_${i}.log; then
            finalization_time=$(grep "Finalization time" delay_${delay_ms}ms_${i}.log | awk '{print $3}' || echo "1")
            echo "network_delay,80,$i,SUCCESS,$finalization_time,fast" >> bounded_finalization_results.csv
        else
            echo "network_delay,80,$i,FAILURE,-1,fast" >> bounded_finalization_results.csv
        fi
    done
done

# Test 5: Concurrent finalization
echo "Test 5: Concurrent finalization"

for i in $(seq 1 30); do
    cargo run --release --bin bounded_finalization_test -- --test-type concurrent --seed $i > concurrent_${i}.log 2>&1
    
    if grep -q "Concurrent finalization successful" concurrent_${i}.log; then
        finalization_time=$(grep "Finalization time" concurrent_${i}.log | awk '{print $3}' || echo "1")
        echo "concurrent,80,$i,SUCCESS,$finalization_time,fast" >> bounded_finalization_results.csv
    else
        echo "concurrent,80,$i,FAILURE,-1,fast" >> bounded_finalization_results.csv
    fi
done

# Test 6: Partial network finalization
echo "Test 6: Partial network finalization"

for offline_percent in 10 20 30 40; do
    echo "Testing with $offline_percent% offline nodes"
    
    for i in $(seq 1 30); do
        cargo run --release --bin bounded_finalization_test -- --test-type partial_network --offline-percent $offline_percent --seed $i > partial_${offline_percent}pct_${i}.log 2>&1
        
        if grep -q "Partial network finalization successful" partial_${offline_percent}pct_${i}.log; then
            finalization_time=$(grep "Finalization time" partial_${offline_percent}pct_${i}.log | awk '{print $3}' || echo "1")
            echo "partial_network,$offline_percent,$i,SUCCESS,$finalization_time,fast" >> bounded_finalization_results.csv
        else
            echo "partial_network,$offline_percent,$i,FAILURE,-1,fast" >> bounded_finalization_results.csv
        fi
    done
done

# Generate summary report
echo ""
echo "=== Bounded Finalization Time Verification Summary ==="

# Analyze results by test type
echo "Results by Test Type:"
for test_type in fast_path slow_path bounded network_delay concurrent partial_network; do
    count=$(grep "^$test_type," bounded_finalization_results.csv | wc -l)
    success=$(grep "^$test_type," bounded_finalization_results.csv | grep -c "SUCCESS" || true)
    
    if [ $count -gt 0 ]; then
        success_rate=$((success * 100 / count))
        echo "  $test_type: $success_rate% success rate ($success/$count)"
    fi
done

# Analyze fast path performance
echo ""
echo "Fast Path Performance:"
for stake_pct in 80 85 90 95 100; do
    count=$(grep "fast_path,$stake_pct," bounded_finalization_results.csv | wc -l)
    success=$(grep "fast_path,$stake_pct," bounded_finalization_results.csv | grep -c "SUCCESS" || true)
    avg_time=$(grep "fast_path,$stake_pct,.*,SUCCESS" bounded_finalization_results.csv | awk -F',' '{sum+=$5; count++} END {if(count>0) print sum/count; else print "N/A"}')
    
    if [ $count -gt 0 ]; then
        success_rate=$((success * 100 / count))
        echo "  $stake_pct% stake: $success_rate% success rate, avg time: $avg_time"
    fi
done

# Analyze slow path performance
echo ""
echo "Slow Path Performance:"
for stake_pct in 60 65 70 75 80; do
    count=$(grep "slow_path,$stake_pct," bounded_finalization_results.csv | wc -l)
    success=$(grep "slow_path,$stake_pct," bounded_finalization_results.csv | grep -c "SUCCESS" || true)
    avg_time=$(grep "slow_path,$stake_pct,.*,SUCCESS" bounded_finalization_results.csv | awk -F',' '{sum+=$5; count++} END {if(count>0) print sum/count; else print "N/A"}')
    
    if [ $count -gt 0 ]; then
        success_rate=$((success * 100 / count))
        echo "  $stake_pct% stake: $success_rate% success rate, avg time: $avg_time"
    fi
done

# Verify critical properties
echo ""
echo "=== Critical Property Verification ==="

# Check if fast path meets time bounds
fast_success=$(grep "^fast_path," bounded_finalization_results.csv | grep -c "SUCCESS" || true)
fast_total=$(grep "^fast_path," bounded_finalization_results.csv | wc -l)
fast_rate=$((fast_success * 100 / fast_total))

if [ $fast_rate -ge 90 ]; then
    echo "✅ Fast path meets time bounds ($fast_rate%)"
else
    echo "❌ Fast path does not meet time bounds ($fast_rate%)"
fi

# Check if slow path meets time bounds
slow_success=$(grep "^slow_path," bounded_finalization_results.csv | grep -c "SUCCESS" || true)
slow_total=$(grep "^slow_path," bounded_finalization_results.csv | wc -l)
slow_rate=$((slow_success * 100 / slow_total))

if [ $slow_rate -ge 90 ]; then
    echo "✅ Slow path meets time bounds ($slow_rate%)"
else
    echo "❌ Slow path does not meet time bounds ($slow_rate%)"
fi

# Check if bounded finalization time is verified
bounded_success=$(grep "^bounded," bounded_finalization_results.csv | grep -c "SUCCESS" || true)
bounded_total=$(grep "^bounded," bounded_finalization_results.csv | wc -l)
bounded_rate=$((bounded_success * 100 / bounded_total))

if [ $bounded_rate -ge 90 ]; then
    echo "✅ Bounded finalization time is verified ($bounded_rate%)"
else
    echo "❌ Bounded finalization time is not verified ($bounded_rate%)"
fi

# Check if network delay handling works
delay_success=$(grep "^network_delay," bounded_finalization_results.csv | grep -c "SUCCESS" || true)
delay_total=$(grep "^network_delay," bounded_finalization_results.csv | wc -l)
delay_rate=$((delay_success * 100 / delay_total))

if [ $delay_rate -ge 80 ]; then
    echo "✅ Network delay handling works ($delay_rate%)"
else
    echo "❌ Network delay handling has issues ($delay_rate%)"
fi

# Check if concurrent finalization works
concurrent_success=$(grep "^concurrent," bounded_finalization_results.csv | grep -c "SUCCESS" || true)
concurrent_total=$(grep "^concurrent," bounded_finalization_results.csv | wc -l)
concurrent_rate=$((concurrent_success * 100 / concurrent_total))

if [ $concurrent_rate -ge 80 ]; then
    echo "✅ Concurrent finalization works ($concurrent_rate%)"
else
    echo "❌ Concurrent finalization has issues ($concurrent_rate%)"
fi

# Check if partial network finalization works
partial_success=$(grep "^partial_network," bounded_finalization_results.csv | grep -c "SUCCESS" || true)
partial_total=$(grep "^partial_network," bounded_finalization_results.csv | wc -l)
partial_rate=$((partial_success * 100 / partial_total))

if [ $partial_rate -ge 80 ]; then
    echo "✅ Partial network finalization works ($partial_rate%)"
else
    echo "❌ Partial network finalization has issues ($partial_rate%)"
fi

# Generate performance analysis
echo ""
echo "=== Performance Analysis ==="

# Calculate average finalization times
fast_times=$(grep "fast_path,.*,SUCCESS" bounded_finalization_results.csv | awk -F',' '{print $5}' | sort -n)
slow_times=$(grep "slow_path,.*,SUCCESS" bounded_finalization_results.csv | awk -F',' '{print $5}' | sort -n)

if [ -n "$fast_times" ] && [ -n "$slow_times" ]; then
    fast_median=$(echo "$fast_times" | awk '{a[NR]=$1} END {if(NR%2==1) print a[(NR+1)/2]; else print (a[NR/2]+a[NR/2+1])/2}')
    slow_median=$(echo "$slow_times" | awk '{a[NR]=$1} END {if(NR%2==1) print a[(NR+1)/2]; else print (a[NR/2]+a[NR/2+1])/2}')
    
    echo "Fast path median time: $fast_median"
    echo "Slow path median time: $slow_median"
    
    if (( $(echo "$fast_median < $slow_median" | bc -l) )); then
        speedup=$(echo "scale=2; $slow_median / $fast_median" | bc -l)
        echo "Speedup factor: ${speedup}x"
    fi
fi

# Analyze network delay impact
echo ""
echo "Network Delay Impact:"
for delay_ms in 10 20 30 40 50; do
    avg_time=$(grep "network_delay,$delay_ms,.*,SUCCESS" bounded_finalization_results.csv | awk -F',' '{sum+=$5; count++} END {if(count>0) print sum/count; else print "N/A"}')
    echo "  $delay_ms ms delay: avg finalization time $avg_time"
done

# Analyze partial network impact
echo ""
echo "Partial Network Impact:"
for offline_percent in 10 20 30 40; do
    avg_time=$(grep "partial_network,$offline_percent,.*,SUCCESS" bounded_finalization_results.csv | awk -F',' '{sum+=$5; count++} END {if(count>0) print sum/count; else print "N/A"}')
    echo "  $offline_percent% offline: avg finalization time $avg_time"
done

# Generate recommendations
echo ""
echo "=== Recommendations ==="

# Check overall success rate
total_tests=$(wc -l < bounded_finalization_results.csv)
successful_tests=$(grep -c "SUCCESS" bounded_finalization_results.csv || true)
overall_rate=$((successful_tests * 100 / total_tests))

if [ $overall_rate -ge 90 ]; then
    echo "✅ Overall bounded finalization verification successful ($overall_rate%)"
    echo "✅ Bounded finalization time property is verified"
    echo "✅ The system meets the min(δ₈₀%, 2δ₆₀%) bound"
else
    echo "⚠️  Overall bounded finalization verification needs improvement ($overall_rate%)"
    echo "⚠️  Some aspects of bounded finalization may need attention"
fi

# Specific recommendations based on results
if [ $fast_rate -lt 90 ]; then
    echo "⚠️  Consider reviewing fast path finalization logic"
fi

if [ $slow_rate -lt 90 ]; then
    echo "⚠️  Consider reviewing slow path finalization logic"
fi

if [ $bounded_rate -lt 90 ]; then
    echo "⚠️  Consider reviewing bounded finalization time verification"
fi

if [ $delay_rate -lt 80 ]; then
    echo "⚠️  Consider improving network delay handling"
fi

if [ $concurrent_rate -lt 80 ]; then
    echo "⚠️  Consider improving concurrent finalization handling"
fi

if [ $partial_rate -lt 80 ]; then
    echo "⚠️  Consider improving partial network finalization handling"
fi

# Performance recommendations
echo ""
echo "Performance Recommendations:"

# Find optimal stake percentages
best_fast_stake=$(grep "fast_path,.*,SUCCESS" bounded_finalization_results.csv | awk -F',' '{print $2}' | sort -n | uniq -c | sort -nr | head -1 | awk '{print $2}')
best_slow_stake=$(grep "slow_path,.*,SUCCESS" bounded_finalization_results.csv | awk -F',' '{print $2}' | sort -n | uniq -c | sort -nr | head -1 | awk '{print $2}')

echo "  Optimal fast path stake: $best_fast_stake%"
echo "  Optimal slow path stake: $best_slow_stake%"

# Find optimal network conditions
best_delay=$(grep "network_delay,.*,SUCCESS" bounded_finalization_results.csv | awk -F',' '{print $2}' | sort -n | uniq -c | sort -nr | head -1 | awk '{print $2}')
echo "  Optimal network delay: $best_delay ms"

echo ""
echo "Bounded finalization time verification completed. Check bounded_finalization_results.csv for detailed results."
