#!/bin/bash
# Alpenglow Rotor Sampling Verification Script
# This script verifies rotor sampling for efficient message dissemination

set -e

echo "=== Alpenglow Rotor Sampling Verification ==="
echo "Timestamp: $(date)"
echo "Testing rotor sampling for efficient message dissemination"
echo ""

# Create results directory
mkdir -p results/rotor_verification
cd results/rotor_verification

# Initialize results file
echo "Test_Type,Node_Count,Fanout,Test_Run,Result,Details" > rotor_results.csv

# Test 1: Basic rotor sampling
echo "Test 1: Basic rotor sampling"

for node_count in 10 20 50 100; do
    echo "Testing with $node_count nodes"
    
    for i in $(seq 1 30); do
        cargo run --release --bin rotor_verification -- --test-type basic --nodes $node_count --seed $i > basic_${node_count}n_${i}.log 2>&1
        
        if grep -q "Rotor sampling successful" basic_${node_count}n_${i}.log; then
            echo "basic,$node_count,3,$i,SUCCESS,Valid rotor sampling" >> rotor_results.csv
        else
            echo "basic,$node_count,3,$i,FAILURE,Invalid rotor sampling" >> rotor_results.csv
        fi
    done
done

# Test 2: Stake-weighted selection
echo "Test 2: Stake-weighted selection"

for node_count in 10 20 50; do
    echo "Testing stake-weighted selection with $node_count nodes"
    
    for i in $(seq 1 30); do
        cargo run --release --bin rotor_verification -- --test-type stake_weighted --nodes $node_count --seed $i > stake_weighted_${node_count}n_${i}.log 2>&1
        
        if grep -q "Stake-weighted selection successful" stake_weighted_${node_count}n_${i}.log; then
            echo "stake_weighted,$node_count,3,$i,SUCCESS,Valid stake-weighted selection" >> rotor_results.csv
        else
            echo "stake_weighted,$node_count,3,$i,FAILURE,Invalid stake-weighted selection" >> rotor_results.csv
        fi
    done
done

# Test 3: Fanout optimization
echo "Test 3: Fanout optimization"

for fanout in 2 3 4 5 6; do
    echo "Testing with fanout $fanout"
    
    for i in $(seq 1 30); do
        cargo run --release --bin rotor_verification -- --test-type fanout --fanout $fanout --seed $i > fanout_${fanout}_${i}.log 2>&1
        
        if grep -q "Fanout optimization successful" fanout_${fanout}_${i}.log; then
            echo "fanout,20,$fanout,$i,SUCCESS,Valid fanout optimization" >> rotor_results.csv
        else
            echo "fanout,20,$fanout,$i,FAILURE,Invalid fanout optimization" >> rotor_results.csv
        fi
    done
done

# Test 4: Message dissemination efficiency
echo "Test 4: Message dissemination efficiency"

for node_count in 10 20 50 100; do
    echo "Testing message dissemination with $node_count nodes"
    
    for i in $(seq 1 30); do
        cargo run --release --bin rotor_verification -- --test-type dissemination --nodes $node_count --seed $i > dissemination_${node_count}n_${i}.log 2>&1
        
        if grep -q "Message dissemination successful" dissemination_${node_count}n_${i}.log; then
            echo "dissemination,$node_count,3,$i,SUCCESS,Valid message dissemination" >> rotor_results.csv
        else
            echo "dissemination,$node_count,3,$i,FAILURE,Invalid message dissemination" >> rotor_results.csv
        fi
    done
done

# Test 5: Network topology adaptation
echo "Test 5: Network topology adaptation"

for topology in mesh star ring tree; do
    echo "Testing with $topology topology"
    
    for i in $(seq 1 30); do
        cargo run --release --bin rotor_verification -- --test-type topology --topology $topology --seed $i > topology_${topology}_${i}.log 2>&1
        
        if grep -q "Topology adaptation successful" topology_${topology}_${i}.log; then
            echo "topology,20,3,$i,SUCCESS,Valid topology adaptation" >> rotor_results.csv
        else
            echo "topology,20,3,$i,FAILURE,Invalid topology adaptation" >> rotor_results.csv
        fi
    done
done

# Test 6: Fault tolerance
echo "Test 6: Fault tolerance"

for fault_percent in 10 20 30 40; do
    echo "Testing with $fault_percent% faulty nodes"
    
    for i in $(seq 1 30); do
        cargo run --release --bin rotor_verification -- --test-type fault_tolerance --fault-percent $fault_percent --seed $i > fault_${fault_percent}pct_${i}.log 2>&1
        
        if grep -q "Fault tolerance successful" fault_${fault_percent}pct_${i}.log; then
            echo "fault_tolerance,20,3,$i,SUCCESS,Valid fault tolerance" >> rotor_results.csv
        else
            echo "fault_tolerance,20,3,$i,FAILURE,Invalid fault tolerance" >> rotor_results.csv
        fi
    done
done

# Test 7: Load balancing
echo "Test 7: Load balancing"

for i in $(seq 1 50); do
    cargo run --release --bin rotor_verification -- --test-type load_balancing --seed $i > load_balancing_${i}.log 2>&1
    
    if grep -q "Load balancing successful" load_balancing_${i}.log; then
        echo "load_balancing,20,3,$i,SUCCESS,Valid load balancing" >> rotor_results.csv
    else
        echo "load_balancing,20,3,$i,FAILURE,Invalid load balancing" >> rotor_results.csv
    fi
done

# Test 8: Scalability
echo "Test 8: Scalability"

for node_count in 100 200 500 1000; do
    echo "Testing scalability with $node_count nodes"
    
    for i in $(seq 1 20); do
        cargo run --release --bin rotor_verification -- --test-type scalability --nodes $node_count --seed $i > scalability_${node_count}n_${i}.log 2>&1
        
        if grep -q "Scalability test successful" scalability_${node_count}n_${i}.log; then
            echo "scalability,$node_count,3,$i,SUCCESS,Valid scalability" >> rotor_results.csv
        else
            echo "scalability,$node_count,3,$i,FAILURE,Invalid scalability" >> rotor_results.csv
        fi
    done
done

# Generate summary report
echo ""
echo "=== Rotor Sampling Verification Summary ==="

# Analyze results by test type
echo "Results by Test Type:"
for test_type in basic stake_weighted fanout dissemination topology fault_tolerance load_balancing scalability; do
    count=$(grep "^$test_type," rotor_results.csv | wc -l)
    success=$(grep "^$test_type," rotor_results.csv | grep -c "SUCCESS" || true)
    
    if [ $count -gt 0 ]; then
        success_rate=$((success * 100 / count))
        echo "  $test_type: $success_rate% success rate ($success/$count)"
    fi
done

# Analyze results by node count
echo ""
echo "Results by Node Count:"
for node_count in 10 20 50 100 200 500 1000; do
    count=$(grep ",$node_count," rotor_results.csv | wc -l)
    success=$(grep ",$node_count," rotor_results.csv | grep -c "SUCCESS" || true)
    
    if [ $count -gt 0 ]; then
        success_rate=$((success * 100 / count))
        echo "  $node_count nodes: $success_rate% success rate ($success/$count)"
    fi
done

# Analyze results by fanout
echo ""
echo "Results by Fanout:"
for fanout in 2 3 4 5 6; do
    count=$(grep ",$fanout," rotor_results.csv | wc -l)
    success=$(grep ",$fanout," rotor_results.csv | grep -c "SUCCESS" || true)
    
    if [ $count -gt 0 ]; then
        success_rate=$((success * 100 / count))
        echo "  Fanout $fanout: $success_rate% success rate ($success/$count)"
    fi
done

# Verify critical properties
echo ""
echo "=== Critical Property Verification ==="

# Check if basic rotor sampling works
basic_success=$(grep "^basic," rotor_results.csv | grep -c "SUCCESS" || true)
basic_total=$(grep "^basic," rotor_results.csv | wc -l)
basic_rate=$((basic_success * 100 / basic_total))

if [ $basic_rate -ge 90 ]; then
    echo "✅ Basic rotor sampling works correctly ($basic_rate%)"
else
    echo "❌ Basic rotor sampling has issues ($basic_rate%)"
fi

# Check if stake-weighted selection works
stake_success=$(grep "^stake_weighted," rotor_results.csv | grep -c "SUCCESS" || true)
stake_total=$(grep "^stake_weighted," rotor_results.csv | wc -l)
stake_rate=$((stake_success * 100 / stake_total))

if [ $stake_rate -ge 90 ]; then
    echo "✅ Stake-weighted selection works correctly ($stake_rate%)"
else
    echo "❌ Stake-weighted selection has issues ($stake_rate%)"
fi

# Check if message dissemination works
dissemination_success=$(grep "^dissemination," rotor_results.csv | grep -c "SUCCESS" || true)
dissemination_total=$(grep "^dissemination," rotor_results.csv | wc -l)
dissemination_rate=$((dissemination_success * 100 / dissemination_total))

if [ $dissemination_rate -ge 90 ]; then
    echo "✅ Message dissemination works correctly ($dissemination_rate%)"
else
    echo "❌ Message dissemination has issues ($dissemination_rate%)"
fi

# Check if fault tolerance works
fault_success=$(grep "^fault_tolerance," rotor_results.csv | grep -c "SUCCESS" || true)
fault_total=$(grep "^fault_tolerance," rotor_results.csv | wc -l)
fault_rate=$((fault_success * 100 / fault_total))

if [ $fault_rate -ge 80 ]; then
    echo "✅ Fault tolerance works correctly ($fault_rate%)"
else
    echo "❌ Fault tolerance has issues ($fault_rate%)"
fi

# Check if scalability works
scalability_success=$(grep "^scalability," rotor_results.csv | grep -c "SUCCESS" || true)
scalability_total=$(grep "^scalability," rotor_results.csv | wc -l)
scalability_rate=$((scalability_success * 100 / scalability_total))

if [ $scalability_rate -ge 80 ]; then
    echo "✅ Scalability works correctly ($scalability_rate%)"
else
    echo "❌ Scalability has issues ($scalability_rate%)"
fi

# Generate performance analysis
echo ""
echo "=== Performance Analysis ==="

# Analyze node count performance
echo "Node Count Performance:"
for node_count in 10 20 50 100 200 500 1000; do
    avg_time=$(grep ".*,$node_count,.*,SUCCESS" rotor_results.csv | awk -F',' '{sum+=$6; count++} END {if(count>0) print sum/count; else print "N/A"}')
    echo "  $node_count nodes: avg time $avg_time"
done

# Analyze fanout performance
echo ""
echo "Fanout Performance:"
for fanout in 2 3 4 5 6; do
    avg_time=$(grep ".*,$fanout,.*,SUCCESS" rotor_results.csv | awk -F',' '{sum+=$6; count++} END {if(count>0) print sum/count; else print "N/A"}')
    echo "  Fanout $fanout: avg time $avg_time"
done

# Analyze topology performance
echo ""
echo "Topology Performance:"
for topology in mesh star ring tree; do
    avg_time=$(grep "topology,$topology,.*,SUCCESS" rotor_results.csv | awk -F',' '{sum+=$6; count++} END {if(count>0) print sum/count; else print "N/A"}')
    echo "  $topology topology: avg time $avg_time"
done

# Generate recommendations
echo ""
echo "=== Recommendations ==="

# Check overall success rate
total_tests=$(wc -l < rotor_results.csv)
successful_tests=$(grep -c "SUCCESS" rotor_results.csv || true)
overall_rate=$((successful_tests * 100 / total_tests))

if [ $overall_rate -ge 90 ]; then
    echo "✅ Overall rotor sampling verification successful ($overall_rate%)"
    echo "✅ Rotor sampling for message dissemination is working correctly"
    echo "✅ The system can handle various network conditions and scales well"
else
    echo "⚠️  Overall rotor sampling verification needs improvement ($overall_rate%)"
    echo "⚠️  Some aspects of rotor sampling may need attention"
fi

# Specific recommendations based on results
if [ $basic_rate -lt 90 ]; then
    echo "⚠️  Consider reviewing basic rotor sampling algorithm"
fi

if [ $stake_rate -lt 90 ]; then
    echo "⚠️  Consider reviewing stake-weighted selection logic"
fi

if [ $dissemination_rate -lt 90 ]; then
    echo "⚠️  Consider reviewing message dissemination efficiency"
fi

if [ $fault_rate -lt 80 ]; then
    echo "⚠️  Consider improving fault tolerance mechanisms"
fi

if [ $scalability_rate -lt 80 ]; then
    echo "⚠️  Consider improving scalability for large networks"
fi

# Performance recommendations
echo ""
echo "Performance Recommendations:"

# Find optimal fanout
best_fanout=$(grep "fanout,.*,SUCCESS" rotor_results.csv | awk -F',' '{print $3}' | sort -n | uniq -c | sort -nr | head -1 | awk '{print $2}')
echo "  Optimal fanout based on tests: $best_fanout"

# Find optimal node count for performance
best_node_count=$(grep ".*,.*,SUCCESS" rotor_results.csv | awk -F',' '{print $2}' | sort -n | uniq -c | sort -nr | head -1 | awk '{print $2}')
echo "  Most tested node count: $best_node_count"

echo ""
echo "Rotor sampling verification completed. Check rotor_results.csv for detailed results."
