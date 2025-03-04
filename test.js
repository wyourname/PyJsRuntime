// function fibonacci(n) {
//     if (n <= 1) return n;
//     return fibonacci(n - 1) + fibonacci(n - 2);
// }

// fibonacci(35);

import { getSig3 } from './sig3_56.js';

function test() {
    console.log('test');
    return getSig3('{"reportCount":1,"subBizId":6428,"taskId":26035}');
}

// test();