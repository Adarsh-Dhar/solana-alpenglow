#!/bin/bash
# Alpenglow Dual-Path Finality Verification Script
# This script verifies the dual-path finality mechanism (fast vs slow path)

set -e

echo "=== Alpenglow Dual-Path Finality Verification ==="
echo "Timestamp: $(date)"
echo "Testing fast path (80% threshold) vs slow path (60% threshold) finality"
echo ""

# Create results directory
mkdir -p results/dual_path_verification
cd results/dual_path_verification

# Initialize results file
echo "Path_Type,Stake_Percent,Test_Run,Result,Finalization_Time,Round_Count" > dual_path_results.csv

# Test Fast Path (80% threshold)
echo "Testing Fast Path finalization (80% threshold)..."

for stake_pct in 80 85 90 95 100; do
    echo "Testing fast path with $stake_pct% stake..."
    
    for i in $(seq 1 100); do
        cargo run --release --bin dual_path_test -- --path fast --threshold 80 --stake-percent $stake_pct --seed $i > fast_${stake_pct}pct_${i}.log 2>&1
        
        if grep -q "Fast path finalization successful" fast_${stake_pct}pct_${i}.log; then
            finalization_time=$(grep "Finalization time" fast_${stake_pct}pct_${i}.log | awk '{print $3}' || echo "1")
            round_count=$(grep "Rounds completed" fast_${stake_pct}pct_${i}.log | awk '{print $3}' || echo "1")
            echo "fast,$stake_pct,$i,SUCCESS,$finalization_time,$round_count" >> dual_path_results.csv
        else
            echo "fast,$stake_pct,$i,FAILURE,-1,0" >> dual_path_results.csv
        fi
    done
done

# Test Slow Path (60% threshold)
echo "Testing Slow Path finalization (60% threshold)..."

for stake_pct in 60 65 70 75 80; do
    echo "Testing slow path with $stake_pct% stake..."
    
    for i in $(seq 1 100); do
        cargo run --release --bin dual_path_test -- --path slow --threshold 60 --stake-percent $stake_pct --seed $i > slow_${stake_pct}pct_${i}.log 2>&1
        
        if grep -q "Slow path finalization successful" slow_${stake_pct}pct_${i}.log; then
            finalization_time=$(grep "Finalization time" slow_${stake_pct}pct_${i}.log | awk '{print $3}' || echo "2")
            round_count=$(grep "Rounds completed" slow_${stake_pct}pct_${i}.log | awk '{print $3}' || echo "2")
            echo "slow,$stake_pct,$i,SUCCESS,$finalization_time,$round_count" >> dual_path_results.csv
        else
            echo "slow,$stake_pct,$i,FAILURE,-1,0" >> dual_path_results.csv
        fi
    done
done

# Test Bounded Finalization Time
echo "Testing Bounded Finalization Time..."

for i in $(seq 1 50); do
    cargo run --release --bin bounded_time_test -- --max-ticks 10 --seed $i > bounded_time_${i}.log 2>&1
    
    if grep -q "Bounded finalization time verified" bounded_time_${i}.log; then
        fast_time=$(grep "Fast path time" bounded_time_${i}.log | awk '{print $4}' || echo "1")
        slow_time=$(grep "Slow path time" bounded_time_${i}.log | awk '{print $4}' || echo "2")
        echo "bounded,100,$i,SUCCESS,$fast_time,1" >> dual_path_results.csv
        echo "bounded,100,$i,SUCCESS,$slow_time,2" >> dual_path_results.csv
    else
        echo "bounded,100,$i,FAILURE,-1,0" >> dual_path_results.csv
    fi
done

# Test Path Selection Logic
echo "Testing Path Selection Logic..."

for stake_pct in 60 70 80 90; do
    echo "Testing path selection with $stake_pct% stake..."
    
    for i in $(seq 1 50); do
        cargo run --release --bin path_selection_test -- --stake-percent $stake_pct --seed $i > path_selection_${stake_pct}pct_${i}.log 2>&1
        
        if grep -q "Correct path selected" path_selection_${stake_pct}pct_${i}.log; then
            selected_path=$(grep "Selected path" path_selection_${stake_pct}pct_${i}.log | awk '{print $3}' || echo "unknown")
            echo "selection,$stake_pct,$i,SUCCESS,1,$selected_path" >> dual_path_results.csv
        else
            echo "selection,$stake_pct,$i,FAILURE,-1,unknown" >> dual_path_results.csv
        fi
    done
done

# Generate summary report
echo ""
echo "=== Dual-Path Finality Verification Summary ==="

# Analyze Fast Path results
echo "Fast Path Results:"
for stake_pct in 80 85 90 95 100; do
    success_count=$(grep "fast,$stake_pct," dual_path_results.csv | grep -c "SUCCESS" || true)
    success_rate=$((success_count * 100 / 100))
    avg_time=$(grep "fast,$stake_pct,.*,SUCCESS" dual_path_results.csv | awk -F',' '{sum+=$5; count++} END {if(count>0) print sum/count; else print "N/A"}')
    echo "  $stake_pct% stake: $success_rate% success rate, avg time: $avg_time"
done

# Analyze Slow Path results
echo ""
echo "Slow Path Results:"
for stake_pct in 60 65 70 75 80; do
    success_count=$(grep "slow,$stake_pct," dual_path_results.csv | grep -c "SUCCESS" || true)
    success_rate=$((success_count * 100 / 100))
    avg_time=$(grep "slow,$stake_pct,.*,SUCCESS" dual_path_results.csv | awk -F',' '{sum+=$5; count++} END {if(count>0) print sum/count; else print "N/A"}')
    echo "  $stake_pct% stake: $success_rate% success rate, avg time: $avg_time"
done

# Analyze Path Selection results
echo ""
echo "Path Selection Results:"
for stake_pct in 60 70 80 90; do
    success_count=$(grep "selection,$stake_pct," dual_path_results.csv | grep -c "SUCCESS" || true)
    success_rate=$((success_count * 100 / 50))
    echo "  $stake_pct% stake: $success_rate% correct path selection"
done

# Verify critical properties
echo ""
echo "=== Critical Property Verification ==="

# Check if fast path works with ≥80% stake
fast_80_success=$(grep "fast,80," dual_path_results.csv | grep -c "SUCCESS" || true)
fast_80_rate=$((fast_80_success * 100 / 100))

if [ $fast_80_rate -ge 90 ]; then
    echo "✅ Fast path works with ≥80% stake ($fast_80_rate%)"
else
    echo "❌ Fast path fails with 80% stake ($fast_80_rate%)"
fi

# Check if slow path works with ≥60% stake
slow_60_success=$(grep "slow,60," dual_path_results.csv | grep -c "SUCCESS" || true)
slow_60_rate=$((slow_60_success * 100 / 100))

if [ $slow_60_rate -ge 90 ]; then
    echo "✅ Slow path works with ≥60% stake ($slow_60_rate%)"
else
    echo "❌ Slow path fails with 60% stake ($slow_60_rate%)"
fi

# Check if fast path is faster than slow path
fast_avg_time=$(grep "fast,.*,SUCCESS" dual_path_results.csv | awk -F',' '{sum+=$5; count++} END {if(count>0) print sum/count; else print "N/A"}')
slow_avg_time=$(grep "slow,.*,SUCCESS" dual_path_results.csv | awk -F',' '{sum+=$5; count++} END {if(count>0) print sum/count; else print "N/A"}')

if [ "$fast_avg_time" != "N/A" ] && [ "$slow_avg_time" != "N/A" ]; then
    if (( $(echo "$fast_avg_time < $slow_avg_time" | bc -l) )); then
        echo "✅ Fast path is faster than slow path ($fast_avg_time vs $slow_avg_time)"
    else
        echo "❌ Fast path is not faster than slow path ($fast_avg_time vs $slow_avg_time)"
    fi
fi

# Check bounded finalization time
bounded_success=$(grep "bounded," dual_path_results.csv | grep -c "SUCCESS" || true)
bounded_rate=$((bounded_success * 100 / 50))

if [ $bounded_rate -ge 90 ]; then
    echo "✅ Bounded finalization time verified ($bounded_rate%)"
else
    echo "❌ Bounded finalization time not verified ($bounded_rate%)"
fi

# Generate performance comparison
echo ""
echo "=== Performance Comparison ==="

# Calculate average times
fast_times=$(grep "fast,.*,SUCCESS" dual_path_results.csv | awk -F',' '{print $5}' | sort -n)
slow_times=$(grep "slow,.*,SUCCESS" dual_path_results.csv | awk -F',' '{print $5}' | sort -n)

if [ -n "$fast_times" ] && [ -n "$slow_times" ]; then
    fast_median=$(echo "$fast_times" | awk '{a[NR]=$1} END {if(NR%2==1) print a[(NR+1)/2]; else print (a[NR/2]+a[NR/2+1])/2}')
    slow_median=$(echo "$slow_times" | awk '{a[NR]=$1} END {if(NR%2==1) print a[(NR+1)/2]; else print (a[NR/2]+a[NR/2+1])/2}')
    
    echo "Fast path median time: $fast_median"
    echo "Slow path median time: $slow_median"
    
    if (( $(echo "$fast_median < $slow_median" | bc -l) )); then
        speedup=$(echo "scale=2; $slow_median / $fast_median" | bc -l)
        echo "Speedup factor: $speedupx"
    fi
fi

echo ""
echo "Dual-path finality verification completed. Check dual_path_results.csv for detailed results."
