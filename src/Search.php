<?php
declare(strict_types = 1);
namespace Williamdes\MariaDBMySQLKBS;

use \Exception;

class Search
{

    /**
     * Loaded data
     *
     * @var mixed
     */
    public static $data;

    /**
     * Data is loaded
     *
     * @var boolean
     */
    public static $loaded = false;

    public const ANY        = -1;
    public const MYSQL      = 1;
    public const MARIADB    = 2;
    public const DS         = DIRECTORY_SEPARATOR;
    public static $DATA_DIR = __DIR__.self::DS."..".self::DS."dist".self::DS;

    /**
     * Load data from disk
     *
     * @return void
     */
    public static function loadData(): void
    {
        if (Search::$loaded === false) {
            $filePath = Search::$DATA_DIR."merged-ultraslim.json";
            $contents = @file_get_contents($filePath);
            if ($contents === false) {
                throw new Exception("$filePath does not exist !");
            }
            Search::$data   = json_decode($contents);
            Search::$loaded = true;
        }
    }

    /**
     * get the first link to doc available
     *
     * @param string $name Name of variable
     * @param int    $type (optional) Type of link Search::MYSQL/Search::MARIADB/Search::ANY
     * @return string
     */
    public static function getByName(string $name, int $type = Search::ANY): string
    {
        self::loadData();
        if (isset(Search::$data->vars->{$name})) {
            $kbEntrys = Search::$data->vars->{$name};
            $kbEntry  = null;
            foreach ($kbEntrys->a as $kbEntry) {
                if ($type === Search::ANY) {
                    return Search::$data->urls[$kbEntry->u]."#".$kbEntry->a;
                } elseif ($type === Search::MYSQL) {
                    if ($kbEntry->t === Search::MYSQL) {
                        return Search::$data->urls[$kbEntry->u]."#".$kbEntry->a;
                    }
                } elseif ($type === Search::MARIADB) {
                    if ($kbEntry->t === Search::MARIADB) {
                        return Search::$data->urls[$kbEntry->u]."#".$kbEntry->a;
                    }
                }
            }

            throw new Exception("$name does not exist for this type of documentation !");
        } else {
            throw new Exception("$name does not exist !");
        }
    }

    /**
     * Return the list of static variables
     *
     * @return array
     */
    public static function getStaticVariables(): array
    {
        return self::getVariablesWithDynamic(false);
    }

    /**
     * Return the list of dynamic variables
     *
     * @return array
     */
    public static function getDynamicVariables(): array
    {
        return self::getVariablesWithDynamic(true);
    }

    /**
     * Return the list of variables having dynamic = $dynamic
     *
     * @param bool $dynamic dynamic=true/dynamic=false
     * @return array
     */
    public static function getVariablesWithDynamic(bool $dynamic): array
    {
        self::loadData();
        $staticVars = array();
        foreach (Search::$data->vars as $name => $var) {
            if (isset($var->d)) {
                if ($var->d === $dynamic) {
                    $staticVars[] = $name;
                }
            }
        }
        return $staticVars;
    }

}
