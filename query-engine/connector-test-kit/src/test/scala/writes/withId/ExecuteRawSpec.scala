package writes

import org.jooq.conf.{ParamType, Settings}
import org.jooq.impl.DSL
import org.jooq.impl.DSL.{field, name, table}
import org.jooq.{Query, SQLDialect}
import org.scalatest.{Matchers, WordSpecLike}
import play.api.libs.json.{JsString, JsValue}
import sangria.util.StringUtil
import util._

class ExecuteRawSpec extends WordSpecLike with Matchers with ApiSpecBase {
  lazy val isMySQL = connectorTag == ConnectorTag.MySqlConnectorTag
  lazy val isPostgres = connectorTag == ConnectorTag.PostgresConnectorTag
  lazy val isSQLite = connectorTag == ConnectorTag.SQLiteConnectorTag

  lazy val dialect = connectorTag match {
    case ConnectorTag.MySqlConnectorTag => SQLDialect.MYSQL_5_7
    case ConnectorTag.PostgresConnectorTag => SQLDialect.POSTGRES
    case ConnectorTag.SQLiteConnectorTag => SQLDialect.SQLITE
    case ConnectorTag.MongoConnectorTag => sys.error("No raw queries for Mongo")
  }

  lazy val sql         = DSL.using(dialect, new Settings().withRenderFormatted(true))
  lazy val modelTable  = table(name("Todo"))
  lazy val idField     = field("id")
  lazy val titleField  = field("title")

  val project = SchemaDsl.fromStringV11() {
    """
      |model Todo {
      |  id String @id @default(cuid())
      |  title String
      |}
    """.stripMargin
  }

  def queryString(query: Query): String = StringUtil.escapeString(query.getSQL(ParamType.INLINED))

  def executeRaw(query: Query): JsValue = {
    server.query(
      s"""mutation {
        |  executeRaw(
        |    query: "${queryString(query)}"
        |  )
        |}
        |""".stripMargin,
      project
    )
  }

  def executeRawThatMustFail(query: Query, errorCode: Int, errorContains: String): JsValue = {
    server.queryThatMustFail(
      s"""mutation {
         |  executeRaw(
         |    query: "${queryString(query)}"
         |  )
         |}
      """.stripMargin,
      project,
      errorCode = errorCode,
      errorContains = errorContains
    )
  }

  override protected def beforeAll(): Unit = {
    super.beforeAll()
    database.setup(project)
  }

  "the simplest query Select 1 should work" in {
    executeRaw(sql.deleteFrom(modelTable))

    val result = server.query(
      """mutation {
        |  executeRaw(
        |    query: "SELECT 1"
        |  )
        |}
          """.stripMargin,
      project,
    )

    val columnName = if (isPostgres) "?column?" else "1"
    result.pathAsJsValue("data.executeRaw") should equal(s"""[{"$columnName":1}]""".parseJson)
  }

  "parameterized queries should work" in {
    executeRaw(sql.deleteFrom(modelTable))

    val query = if (isPostgres) {
      """mutation {
        |  executeRaw(
        |    query: "SELECT ($1)::text"
        |    parameters: "[\"foo\"]",
        |  )
        |}
        |""".stripMargin
    } else {
      """mutation {
        |  executeRaw(
        |    query: "SELECT ?"
        |    parameters: "[\"foo\"]",
        |  )
        |}
        |""".stripMargin
    }
    val result = server.query(query, project)
    val columnName = if (isPostgres) "text" else "?"
    result.pathAsJsValue("data.executeRaw") should equal(s"""[{"$columnName":"foo"}]""".parseJson)
  }

  "querying model tables should work" in {
    executeRaw(sql.deleteFrom(modelTable))

    val res = server.query(
      s"""mutation {
        |   createTodo(data: { title: "title1" }) { id }
        |}
        |""".stripMargin,
      project
    )

    val id = res.pathAsString("data.createTodo.id")
    val result = executeRaw(sql.select().from(modelTable))
    result.pathAsJsValue("data.executeRaw") should equal(s"""[{"id":"$id","title":"title1"}]""".parseJson)
  }

  "inserting into a model table should work" in {
    executeRaw(sql.deleteFrom(modelTable))

    val query = sql
      .insertInto(modelTable)
      .columns(idField, titleField)
      .values("id1", "title1")
      .values("id2", "title2")

    executeRaw(query).pathAsJsValue("data.executeRaw") should equal("2".parseJson)

    val readResult = executeRaw(sql.select().from(modelTable))

    readResult.pathAsJsValue("data.executeRaw") should equal(
      s"""[{"id":"id1","title":"title1"},{"id":"id2","title":"title2"}]""".parseJson)
  }

  "querying model tables with alias should work" in {
    executeRaw(sql.deleteFrom(modelTable))

    server.query(
      s"""mutation {
         |   createTodo(data: { title: "title1" }) { id }
         |}
         |""".stripMargin,
      project
    )

    val result = executeRaw(sql.select(titleField.as("aliasedTitle")).from(modelTable))
    result.pathAsJsValue("data.executeRaw") should equal(s"""[{"aliasedTitle":"title1"}]""".parseJson)
  }

  "querying the same column name twice but aliasing it should work" in {
    executeRaw(sql.deleteFrom(modelTable))

    server.query(
      s"""mutation {
         |   createTodo(data: { title: "title1" }) { id }
         |}
         |""".stripMargin,
      project
    )

    val result = executeRaw(sql.select(titleField.as("ALIASEDTITLE"), titleField).from(modelTable))

    result.pathAsJsValue("data.executeRaw") should equal(
      s"""[{"ALIASEDTITLE":"title1","$titleField":"title1"}]""".parseJson)
  }

  "postgres arrays should work" in {
    executeRaw(sql.deleteFrom(modelTable))

    if (isPostgres) {
      val query =
        """
          |SELECT
          |    array_agg(columnInfos.attname) as postgres_array
          |FROM
          |    pg_attribute columnInfos;
        """.stripMargin

      val result = server.query(
        s"""mutation {
           |  executeRaw(
           |    query: "${StringUtil.escapeString(query)}"
           |  )
           |}
        """.stripMargin,
        project
      )

      val postgresArray = result.pathAsJsArray("data.executeRaw").value.head.pathAsJsArray("postgres_array").value
      postgresArray should not(be(empty))
      val allAreStrings = postgresArray.forall {
        case _: JsString => true
        case _           => false
      }
      allAreStrings should be(true)
    }
  }

  "syntactic errors should bubble through to the user" in {
    executeRaw(sql.deleteFrom(modelTable))

    val (errorCode, errorContains) = () match {
      case _ if isPostgres => (0, "error at end of input")
      case _ if isMySQL    => (1064, "check the manual that corresponds to your MySQL server version for the right syntax to use near")
      case _ if isSQLite   => (1, "incomplete input")
    }

    server.queryThatMustFail(
      s"""mutation {
         |  executeRaw(
         |    query: "Select * from "
         |  )
         |}
      """.stripMargin,
      project,
      errorCode = errorCode,
      errorContains = errorContains
    )
  }

  "other errors should also bubble through to the user" in {
    executeRaw(sql.deleteFrom(modelTable))

    val res = server.query(
      s"""mutation {
         |   createTodo(data: { title: "title1" }) { id }
         |}
         |""".stripMargin,
      project
    )

    val id = res.pathAsString("data.createTodo.id")

    val (errorCode, errorContains) = () match {
      case _ if isPostgres => (0, "duplicate key value violates unique constraint")
      case _ if isMySQL    => (1062, "Duplicate entry")
      case _ if isSQLite   => (19, "Abort due to constraint violation (UNIQUE constraint failed: Todo.id)")
    }

    executeRawThatMustFail(
      sql.insertInto(modelTable).columns(idField, titleField).values(id, "irrelevant"),
      errorCode = errorCode,
      errorContains = errorContains
    )
  }
}

