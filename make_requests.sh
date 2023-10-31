a=0
#Iterate the loop until a less than 10
while [ $a -lt 100 ]; do
    curl --location --request GET 'localhost:4222/files' \
        --header 'Content-Type: text/plain' \
        --data 'Hello' \
        -i

    curl --location --request GET 'localhost:4222/echo/abc' \
        --header 'Content-Type: text/plain' \
        --data 'Hello' \
        -i

    curl --location --request GET 'localhost:4222' \
        --header 'Content-Type: text/plain' \
        --data 'Hello' \
        -i

    curl --location --request GET 'localhost:4222/user-agent' \
        --header 'Content-Type: text/plain' \
        --data 'Hello' \
        -i

    # shellcheck disable=SC2003
    a=$(expr $a + 1)
done
