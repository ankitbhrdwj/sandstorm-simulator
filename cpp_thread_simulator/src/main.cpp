#include <iostream>
#include <random>
#include <thread>
#include <map>
#include<vector>

using namespace std;

std::map<uint64_t, uint64_t> dataMap;

// "busy sleep" while suggesting that other threads run
// for a small amount of time
void little_sleep()
{
    std::thread::id this_id = std::this_thread::get_id();
    std::random_device rd;
    std::default_random_engine generator(rd());
    std::uniform_int_distribution<uint64_t> distribution(0,0xFFFFFFFFFFFFFFFF);
    uint64_t counter = 0;
    while(true) {
        uint64_t randKey = distribution(generator);
        cout << "Key: " << randKey << endl;
        if (dataMap.find(randKey) != dataMap.end()) {
            cout << "Found a value for key: " << randKey << endl;
        }
        if(counter % 10000 == 0) {
            cout << "Current thread is: " << this_id << endl;
            std::this_thread::yield();
        }
    }
}

int main()
{
    std::random_device rd;
    std::default_random_engine generator(rd());
    std::uniform_int_distribution<uint64_t> distribution(0,0xFFFFFFFFFFFFFFFF);

    size_t mapSize = 250000;
    size_t i=0;
    while(i < mapSize) {
        dataMap[distribution(generator)] =  distribution(generator);
        if (i%10000 == 0) {
            cout << "Inserted a value" << endl;
        }
        i+=1;
    }

    vector<thread> threadPool;
    size_t numThreads = 10;
    for(i=0; i<numThreads; i++) {
        cout << "Inserting thread at " << i << endl;
        threadPool.push_back(thread(little_sleep));
    }

    for(i=0; i<numThreads; i++) {
        threadPool[i].join();
    }

    return 0;
}